//! Directory indexes (`$I30`): the INDX records of a directory's `$INDEX_ALLOCATION` attribute
//! named `$I30`, the entries in use in them (each holds a file's `$FILE_NAME`: its name, its
//! parent, its times and sizes as of the name's last update), and the entries left in their
//! slack, past the entries in use: names of files deleted or renamed out of the directory, and
//! copies of entries moved when the index was rebalanced.
//!
//! An INDX record (usually 4096 bytes) starts with its signature, its update sequence array (the
//! last two bytes of each 512-byte sector are kept there and replaced by a check value), its log
//! sequence number and its VCN; then a node header at 0x18: where the entries start, how many
//! bytes they take, how many the record has for them (all relative to 0x18), and whether the
//! node has children. Each entry is the file's MFT reference, the entry's length, the key's
//! length, flags (a subnode, the last entry), the key (`$FILE_NAME`) and, with a subnode, the
//! subnode's VCN in its last 8 bytes.
//!
//! The slack (from the end of the entries in use to the end of the space the record has for
//! them) is scanned at every 8-byte boundary, where entries start: for a whole entry, else for
//! a `$FILE_NAME` alone (its entry header written over); at its start, also for a `$FILE_NAME`
//! whose first 4 or 8 bytes (its parent) the end of the entries in use wrote over. What is taken
//! for one must look like one Windows writes: a name of valid UTF-16 without control characters
//! or path separators, times from 1980 to 2100, a known namespace and known file attribute
//! flags. A parent whose MFT entry number is past 32 bits (NTFS numbers fewer entries) is the
//! bytes of something written over it, such as the end of an entry header, and is left out.

use crate::attribute::FileAttributeFlags;
use crate::attribute::x30::FileNamespace;
use crate::ntfs::apply_update_sequence_array_fixups_in_place_with;

use num_traits::FromPrimitive;
use serde::Serialize;
use winstructs::ntfs::mft_reference::MftReference;

/// An INDX record's signature.
pub const INDX_SIGNATURE: &[u8; 4] = b"INDX";
/// The size of a sector the update sequence array protects.
const SECTOR: usize = 512;
/// Where an INDX record's node header is.
const NODE_HEADER: usize = 0x18;
/// The size of an index entry's header (before its key).
const ENTRY_HEADER: usize = 0x10;
/// The size of a `$FILE_NAME` before its name.
const FILE_NAME_FIXED: usize = 0x42;
/// An entry that has a subnode (its VCN in the entry's last 8 bytes).
const ENTRY_SUBNODE: u32 = 0x1;
/// The last entry of a node (no key).
const ENTRY_LAST: u32 = 0x2;
/// The largest INDX record taken (Windows writes 4096 bytes).
const MAX_RECORD: usize = 64 * 1024;
/// Times taken as a `$FILE_NAME`'s: 1980-01-01 to 2100-01-01 (FILETIMEs).
const EARLIEST: u64 = 119_600_064_000_000_000;
const LATEST: u64 = 157_469_184_000_000_000;
/// The file attribute flags Windows defines (and that a `$FILE_NAME` holds).
const KNOWN_FLAGS: u32 = 0x005f_ffff | 0x1000_0000 | 0x2000_0000;

/// An INDX record of a directory index.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndexRecord {
    /// Where it starts in the data read.
    pub offset: u64,
    /// Its size (its node header's allocated size, and the 0x18 bytes before it).
    pub size: usize,
    /// Its log sequence number.
    pub lsn: u64,
    /// Its VCN in the index.
    pub vcn: u64,
    /// Whether its node has children (its entries have subnodes).
    pub has_children: bool,
    /// Whether each sector held the update sequence value (or the record was saved with the
    /// array already applied): `false` for a record written in part, whose sectors are read as
    /// they are.
    pub valid_fixup: bool,
    /// Its entries in use, in their order.
    pub entries: Vec<IndexEntry>,
    /// The entries found in its slack.
    pub slack: Vec<IndexEntry>,
    /// What keeps parts of it from being read.
    pub problems: Vec<String>,
}

/// An index entry: a file's name in the directory, with its `$FILE_NAME`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndexEntry {
    /// Where it starts in the data read: its header, or for an entry found in slack by its
    /// `$FILE_NAME` alone, that `$FILE_NAME`.
    pub offset: u64,
    /// The file's MFT entry and sequence number; `None` for an entry found in slack whose
    /// header was written over.
    pub file_reference: Option<MftReference>,
    /// Its flags (0x1: a subnode, 0x2: the last entry); `None` without its header.
    pub flags: Option<u32>,
    /// Its subnode's VCN.
    pub subnode_vcn: Option<u64>,
    /// Whether it was found in the record's slack.
    pub from_slack: bool,
    /// For an entry found in slack: whether an entry in use in the same index has the same
    /// name, file and times (a copy left when entries moved).
    pub copy_of_entry_in_use: bool,
    /// The file's name.
    pub name: String,
    /// The name's namespace.
    pub namespace: FileNamespace,
    /// The directory's MFT entry and sequence number; `None` for an entry in the slack whose
    /// first bytes were written over (at the start of the slack, or with an entry number past
    /// 32 bits).
    pub parent: Option<MftReference>,
    /// When the file was made (FILETIME).
    pub created: u64,
    /// When its content was last changed (FILETIME).
    pub modified: u64,
    /// When its MFT record was last changed (FILETIME).
    pub mft_modified: u64,
    /// When it was last read (FILETIME).
    pub accessed: u64,
    /// The size allocated for it.
    pub allocated_size: u64,
    /// Its size.
    pub real_size: u64,
    /// Its file attribute flags.
    pub file_flags: FileAttributeFlags,
    /// Its reparse point tag (or extended attributes' size).
    pub reparse_value: u32,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

fn reference(b: &[u8], at: usize) -> Option<MftReference> {
    let raw = u64_at(b, at)?;
    Some(MftReference::new(
        raw & 0x0000_ffff_ffff_ffff,
        (raw >> 48) as u16,
    ))
}

/// The `$FILE_NAME` at the start of `key`, when it looks like one Windows writes: the entry
/// fields but its location and header are left empty.
fn file_name(key: &[u8], strict_slack: bool) -> Option<IndexEntry> {
    let name_length = usize::from(*key.get(0x40)?);
    let namespace = FileNamespace::from_u8(*key.get(0x41)?)?;
    let name_bytes = key.get(FILE_NAME_FIXED..FILE_NAME_FIXED + name_length * 2)?;
    if name_length == 0 {
        return None;
    }
    let times = [8, 16, 24, 32].map(|at| u64_at(key, at).unwrap_or(0));
    if strict_slack && times.iter().any(|t| !(EARLIEST..=LATEST).contains(t)) {
        return None;
    }
    let flags = u32_at(key, 0x38)?;
    if strict_slack && flags & !KNOWN_FLAGS != 0 {
        return None;
    }
    let units: Vec<u16> = name_bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|u| u16::from_le_bytes(*u))
        .collect();
    let name = String::from_utf16(&units).ok()?;
    if strict_slack
        && name
            .chars()
            .any(|c| c < ' ' || c == '/' || c == '\\' || c == '\u{fffd}')
    {
        return None;
    }
    Some(IndexEntry {
        offset: 0,
        file_reference: None,
        flags: None,
        subnode_vcn: None,
        from_slack: false,
        copy_of_entry_in_use: false,
        name,
        namespace,
        // NTFS numbers at most 2^32 - 1 MFT entries: a larger one is bytes written over the parent
        parent: reference(key, 0).filter(|p| p.entry <= u64::from(u32::MAX)),
        created: times[0],
        modified: times[1],
        mft_modified: times[2],
        accessed: times[3],
        allocated_size: u64_at(key, 0x28)?,
        real_size: u64_at(key, 0x30)?,
        file_flags: FileAttributeFlags::from_bits_retain(flags),
        reparse_value: u32_at(key, 0x3c)?,
    })
}

/// The entry whose header starts at `at` in `record`, when it is a whole entry with a
/// `$FILE_NAME` key that looks like one Windows writes, and how many bytes it takes.
#[derive(Clone, Copy)]
struct EntryLayout {
    length: usize,
    key_length: usize,
    flags: u32,
}

/// The structurally valid layout of the entry at `at`. A subnode VCN is stored after the key,
/// so its eight bytes must not overlap the key.
fn entry_layout(record: &[u8], at: usize, end: usize) -> Option<EntryLayout> {
    let length = usize::from(u16_at(record, at + 8)?);
    let key_length = usize::from(u16_at(record, at + 10)?);
    let flags = u32_at(record, at + 12)?;
    let subnode = if flags & ENTRY_SUBNODE != 0 { 8 } else { 0 };
    let keyed = flags & ENTRY_LAST == 0;
    let minimum = if keyed {
        ENTRY_HEADER + FILE_NAME_FIXED + 2 + subnode
    } else {
        ENTRY_HEADER + subnode
    };
    if length < minimum
        || !length.is_multiple_of(8)
        || at.checked_add(length)? > end
        || flags & !(ENTRY_SUBNODE | ENTRY_LAST) != 0
        || (keyed && key_length < FILE_NAME_FIXED + 2)
        || (!keyed && key_length != 0)
        || ENTRY_HEADER.checked_add(key_length)?.checked_add(subnode)? > length
    {
        return None;
    }
    Some(EntryLayout {
        length,
        key_length,
        flags,
    })
}

fn entry_at(
    record: &[u8],
    at: usize,
    end: usize,
    strict_slack: bool,
) -> Option<(IndexEntry, usize)> {
    let layout = entry_layout(record, at, end)?;
    if layout.flags & ENTRY_LAST != 0 {
        return None;
    }
    let key = record.get(at + ENTRY_HEADER..at + ENTRY_HEADER + layout.key_length)?;
    let mut entry = file_name(key, strict_slack)?;
    entry.file_reference = Some(reference(record, at)?);
    entry.flags = Some(layout.flags);
    if layout.flags & ENTRY_SUBNODE != 0 {
        entry.subnode_vcn = u64_at(record, at + layout.length - 8);
    }
    Some((entry, layout.length))
}

/// The INDX records in `data` (an `$I30` stream, or other data holding INDX records), each with
/// its entries in use and the entries in its slack. Records are looked for at every 512-byte
/// boundary.
#[must_use]
pub fn index_records(data: &[u8]) -> Vec<IndexRecord> {
    let mut records = Vec::new();
    let mut at = 0;
    while at + NODE_HEADER + 16 <= data.len() {
        match data.get(at..at + 4) {
            Some(sig) if sig == INDX_SIGNATURE => {}
            _ => {
                at += SECTOR;
                continue;
            }
        }
        match index_record(&data[at..], at as u64) {
            Some(record) => {
                at += record.size.next_multiple_of(SECTOR);
                records.push(record);
            }
            None => at += SECTOR,
        }
    }
    mark_copies(&mut records);
    records
}

/// The INDX record at the start of `data` (at `offset` in the data read), when its header holds.
#[must_use]
pub fn index_record(data: &[u8], offset: u64) -> Option<IndexRecord> {
    let usa_offset = u16_at(data, 4)?;
    let usa_count = u16_at(data, 6)?;
    let usa_end = usize::from(usa_offset).checked_add(usize::from(usa_count).checked_mul(2)?)?;
    let allocated = u32_at(data, NODE_HEADER + 8)? as usize;
    let size = NODE_HEADER + allocated;
    if !size.is_multiple_of(SECTOR)
        || size > MAX_RECORD
        || usize::from(usa_count) != size / SECTOR + 1
        || usa_end > SECTOR - 2
        || size > data.len()
    {
        return None;
    }
    let mut record = data[..size].to_vec();
    let mut problems = Vec::new();
    // a sector that ends neither with the update sequence value nor with the bytes the array
    // keeps for it was written in part; one that ends with the kept bytes was saved with the
    // array already applied (as some tools export records), which leaves it as it is
    let mut torn = Vec::new();
    let applied =
        apply_update_sequence_array_fixups_in_place_with(&mut record, usa_offset, usa_count, |m| {
            if m.end_of_sector_bytes != m.replacement_bytes {
                torn.push((m.sector_idx, m.end_of_sector_bytes));
            }
        });
    // The shared helper applies replacement bytes after reporting a mismatch so callers can do
    // best-effort structural parsing. For a genuinely torn sector, restore the bytes found in the
    // evidence: stale USA bytes must not silently replace them.
    for (sector, bytes) in &torn {
        let end = (*sector + 1) * SECTOR;
        record[end - 2..end].copy_from_slice(bytes);
    }
    if let Err(e) = &applied {
        problems.push(format!("its update sequence array can't be applied: {e}"));
    }
    if !torn.is_empty() {
        let sectors: Vec<String> = torn.iter().map(|(sector, _)| sector.to_string()).collect();
        problems.push(format!(
            "sectors {} don't end with the update sequence value: the record was written in part",
            sectors.join(", ")
        ));
    }
    let valid_fixup = applied.is_ok() && torn.is_empty();
    let entries_offset = u32_at(&record, NODE_HEADER)? as usize;
    let used = u32_at(&record, NODE_HEADER + 4)? as usize;
    let node_flags = u32_at(&record, NODE_HEADER + 12)?;
    let first = NODE_HEADER + entries_offset;
    let used_end = (NODE_HEADER + used).min(size);
    let mut index = IndexRecord {
        offset,
        size,
        lsn: u64_at(&record, 8)?,
        vcn: u64_at(&record, 16)?,
        has_children: node_flags & 1 != 0,
        valid_fixup,
        entries: Vec::new(),
        slack: Vec::new(),
        problems,
    };
    if first < NODE_HEADER + 16 || first > used_end {
        index.problems.push(format!(
            "its entries start at 0x{entries_offset:x}, past the 0x{used:x} bytes they take"
        ));
    } else {
        entries_in_use(&record, first, used_end, &mut index);
    }
    // the slack: from the end of the entries in use to the end of the record
    let slack_start = used_end.max(first).next_multiple_of(8);
    slack_entries(&record, used_end.max(first), slack_start, size, &mut index);
    Some(index)
}

/// The entries in use: from `first`, until the last entry.
fn entries_in_use(record: &[u8], first: usize, end: usize, index: &mut IndexRecord) {
    let mut at = first;
    while at + ENTRY_HEADER <= end {
        let length = usize::from(u16_at(record, at + 8).unwrap_or(0));
        let Some(layout) = entry_layout(record, at, end) else {
            index.problems.push(format!(
                "the entry at 0x{at:x} (of {length} bytes) has an invalid layout: the entries after it are not read"
            ));
            return;
        };
        if layout.flags & ENTRY_LAST != 0 {
            return;
        }
        match entry_at(record, at, end, false) {
            Some((mut entry, len)) => {
                entry.offset = index.offset + at as u64;
                index.entries.push(entry);
                at += len;
            }
            None => {
                index.problems.push(format!(
                    "the entry at 0x{at:x} (of {} bytes) has a key that can't be read: it was skipped",
                    layout.length
                ));
                at += layout.length;
            }
        }
    }
    index
        .problems
        .push("its entries end without a last entry".to_owned());
}

/// The entries in the slack `[start, end)`: whole entries, else `$FILE_NAME`s whose entry
/// header was written over; at `used` (where the entries in use end), also a `$FILE_NAME` whose
/// parent they wrote over.
fn slack_entries(record: &[u8], used: usize, start: usize, end: usize, index: &mut IndexRecord) {
    let mut at = start;
    // a $FILE_NAME whose times start where the slack does, or 4 bytes into it
    for times in [used, used + 4] {
        let Some(rest) = record.get(times..end) else {
            continue;
        };
        let mut key = vec![0u8; 8];
        key.extend(rest);
        if let Some(mut entry) = file_name(&key, true) {
            let first = times - 8;
            entry.parent = None;
            entry.offset = index.offset + first as u64;
            entry.from_slack = true;
            let len = FILE_NAME_FIXED + entry.name.encode_utf16().count() * 2;
            index.slack.push(entry);
            at = at.max((first + len).next_multiple_of(8));
            break;
        }
    }
    while at + FILE_NAME_FIXED + 2 <= end {
        if let Some((mut entry, len)) = entry_at(record, at, end, true) {
            entry.offset = index.offset + at as u64;
            entry.from_slack = true;
            index.slack.push(entry);
            at += len;
            continue;
        }
        if let Some(mut entry) = record.get(at..end).and_then(|key| file_name(key, true)) {
            entry.offset = index.offset + at as u64;
            entry.from_slack = true;
            let len = FILE_NAME_FIXED + entry.name.encode_utf16().count() * 2;
            index.slack.push(entry);
            at += len.next_multiple_of(8);
            continue;
        }
        at += 8;
    }
}

/// Marks the slack entries that copy an entry in use of the same directory index: the same name,
/// namespace, file (when known), parent and times. A parent missing because the entry header was
/// overwritten is inferred only when the record identifies one unambiguous directory.
fn mark_copies(records: &mut [IndexRecord]) {
    type Identity = (String, u8, [u64; 4], MftReference);
    type FullIdentity = (Identity, MftReference);
    let identity = |e: &IndexEntry, parent: MftReference| -> Identity {
        (
            e.name.clone(),
            e.namespace as u8,
            [e.created, e.modified, e.mft_modified, e.accessed],
            parent,
        )
    };
    let in_use_without_file: std::collections::HashSet<Identity> = records
        .iter()
        .flat_map(|r| &r.entries)
        .filter_map(|e| e.parent.map(|parent| identity(e, parent)))
        .collect();
    let in_use: std::collections::HashSet<FullIdentity> = records
        .iter()
        .flat_map(|r| &r.entries)
        .filter_map(|e| Some((identity(e, e.parent?), e.file_reference?)))
        .collect();
    for record in records {
        let parents: std::collections::HashSet<MftReference> = record
            .entries
            .iter()
            .chain(&record.slack)
            .filter_map(|e| e.parent)
            .collect();
        let inferred_parent = (parents.len() == 1)
            .then(|| parents.iter().next().copied())
            .flatten();
        for entry in &mut record.slack {
            let Some(parent) = entry.parent.or(inferred_parent) else {
                continue;
            };
            let key = identity(entry, parent);
            entry.copy_of_entry_in_use = match entry.file_reference {
                Some(file) => in_use.contains(&(key, file)),
                None => in_use_without_file.contains(&key),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `$FILE_NAME` for `name` under directory 5, its times `t`.
    fn key(name: &str, t: u64) -> Vec<u8> {
        key_under(name, t, 5)
    }

    /// A `$FILE_NAME` for `name` under `parent`, its times `t`.
    fn key_under(name: &str, t: u64, parent: u64) -> Vec<u8> {
        let mut k = vec![0u8; FILE_NAME_FIXED];
        k[..8].copy_from_slice(&(parent | (5u64 << 48)).to_le_bytes());
        for at in [8, 16, 24, 32] {
            k[at..at + 8].copy_from_slice(&t.to_le_bytes());
        }
        k[0x28..0x30].copy_from_slice(&4096u64.to_le_bytes());
        k[0x30..0x38].copy_from_slice(&10u64.to_le_bytes());
        k[0x38..0x3c].copy_from_slice(&0x20u32.to_le_bytes());
        k[0x40] = u8::try_from(name.encode_utf16().count()).unwrap_or(0);
        k[0x41] = 1;
        k.extend(name.encode_utf16().flat_map(u16::to_le_bytes));
        k
    }

    /// An index entry for file `entry` with `key`.
    fn entry(file: u64, key: &[u8], flags: u32) -> Vec<u8> {
        let mut e = vec![0u8; ENTRY_HEADER];
        e[..8].copy_from_slice(&(file | (1u64 << 48)).to_le_bytes());
        e[10..12].copy_from_slice(&u16::try_from(key.len()).unwrap_or(0).to_le_bytes());
        e[12..16].copy_from_slice(&flags.to_le_bytes());
        e.extend(key);
        e.resize(e.len().next_multiple_of(8), 0);
        let len = u16::try_from(e.len()).unwrap_or(0);
        e[8..10].copy_from_slice(&len.to_le_bytes());
        e
    }

    /// A 4096-byte INDX record with `entries` in use and `slack` after them, its fixups applied
    /// (`torn`: one sector without the update sequence value).
    fn record(entries: &[Vec<u8>], slack: &[u8], torn: bool) -> Vec<u8> {
        let mut r = vec![0u8; 4096];
        r[..4].copy_from_slice(INDX_SIGNATURE);
        r[4..6].copy_from_slice(&0x28u16.to_le_bytes());
        r[6..8].copy_from_slice(&9u16.to_le_bytes());
        r[16..24].copy_from_slice(&3u64.to_le_bytes());
        let first = 0x40usize;
        let mut at = NODE_HEADER + first;
        for e in entries {
            r[at..at + e.len()].copy_from_slice(e);
            at += e.len();
        }
        let last = {
            let mut l = vec![0u8; ENTRY_HEADER];
            l[8..10].copy_from_slice(&16u16.to_le_bytes());
            l[12..16].copy_from_slice(&ENTRY_LAST.to_le_bytes());
            l
        };
        r[at..at + 16].copy_from_slice(&last);
        at += 16;
        r[at..at + slack.len()].copy_from_slice(slack);
        let set =
            |r: &mut Vec<u8>, at: usize, v: u32| r[at..at + 4].copy_from_slice(&v.to_le_bytes());
        set(&mut r, NODE_HEADER, u32::try_from(first).unwrap_or(0));
        set(
            &mut r,
            NODE_HEADER + 4,
            u32::try_from(at - NODE_HEADER).unwrap_or(0),
        );
        set(
            &mut r,
            NODE_HEADER + 8,
            4096 - u32::try_from(NODE_HEADER).unwrap_or(0),
        );
        // the update sequence array: value 0x0102, the sectors' last bytes kept in it
        r[0x28..0x2a].copy_from_slice(&[1, 2]);
        for sector in 0..8 {
            let end = (sector + 1) * SECTOR - 2;
            let kept = [r[end], r[end + 1]];
            r[0x2a + sector * 2..0x2c + sector * 2].copy_from_slice(&kept);
            let value = if torn && sector == 5 { [9, 9] } else { [1, 2] };
            r[end..end + 2].copy_from_slice(&value);
        }
        r
    }

    const WHEN: u64 = 132_888_278_320_000_000;

    #[test]
    fn entries_in_use_and_in_slack() {
        let live = [
            entry(64, &key("kept.txt", WHEN), 0),
            entry(65, &key("日本語.txt", WHEN), 0),
        ];
        // slack: a whole deleted entry, a $FILE_NAME whose header was written over, a copy of a
        // live entry, and noise
        let mut slack = entry(70, &key("deleted.exe", WHEN + 1), 0);
        let mut over = vec![0xffu8; ENTRY_HEADER];
        over.extend(key("headerless.doc", WHEN + 2));
        over.resize(over.len().next_multiple_of(8), 0);
        slack.extend(over);
        slack.extend(entry(64, &key("kept.txt", WHEN), 0));
        slack.extend([0x41u8; 64]);
        let records = index_records(&record(&live, &slack, false));
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert!(r.valid_fixup && r.problems.is_empty(), "{:?}", r.problems);
        assert_eq!(r.vcn, 3);
        let names: Vec<&str> = r.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["kept.txt", "日本語.txt"]);
        assert_eq!(r.entries[0].file_reference.map(|f| f.entry), Some(64));
        assert_eq!(r.entries[0].parent.map(|p| p.entry), Some(5));
        assert_eq!(
            (r.entries[0].real_size, r.entries[0].allocated_size),
            (10, 4096)
        );
        let slack: Vec<(&str, Option<u64>, bool)> = r
            .slack
            .iter()
            .map(|e| {
                (
                    e.name.as_str(),
                    e.file_reference.map(|f| f.entry),
                    e.copy_of_entry_in_use,
                )
            })
            .collect();
        assert_eq!(
            slack,
            [
                ("deleted.exe", Some(70), false),
                ("headerless.doc", None, false),
                ("kept.txt", Some(64), true),
            ]
        );
        assert_eq!(r.slack[0].created, WHEN + 1);
        assert!(r.slack.iter().all(|e| e.from_slack));
    }

    #[test]
    fn damaged_records() {
        // a sector written in part: read as it is, said so
        let live = [entry(64, &key("kept.txt", WHEN), 0)];
        let r = &index_records(&record(&live, &[], true))[0];
        assert!(!r.valid_fixup);
        assert_eq!(r.entries.len(), 1);
        assert_eq!(
            r.problems,
            ["sectors 5 don't end with the update sequence value: the record was written in part"]
        );
        // saved with the array already applied: the sectors end with the bytes it keeps
        let mut saved = record(&live, &[], false);
        for sector in 0..8 {
            let end = (sector + 1) * SECTOR - 2;
            let kept = [saved[0x2a + sector * 2], saved[0x2b + sector * 2]];
            saved[end..end + 2].copy_from_slice(&kept);
        }
        let r = &index_records(&saved)[0];
        assert!(r.valid_fixup && r.problems.is_empty(), "{:?}", r.problems);
        assert_eq!(r.entries.len(), 1);
        // a key that is not a $FILE_NAME: its structurally valid entry is skipped, without
        // hiding valid entries that follow it
        let mut bad = key("x", WHEN);
        bad[0x41] = 9;
        let live = [entry(64, &bad, 0), entry(65, &key("after.txt", WHEN), 0)];
        let r = &index_records(&record(&live, &[], false))[0];
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].name, "after.txt");
        assert!(
            r.problems[0].starts_with("the entry at 0x58"),
            "{:?}",
            r.problems
        );
        // not INDX records; a header that doesn't hold
        assert!(index_records(&[0u8; 8192]).is_empty());
        let mut r = record(&[], &[], false);
        r[6] = 3;
        assert!(index_records(&r).is_empty());
        // times out of range, control characters: not taken for names
        assert!(file_name(&key("ok", WHEN), true).is_some());
        assert!(file_name(&key("ok", 1), true).is_none());
        assert!(file_name(&key("a\u{1}b", WHEN), true).is_none());
        assert!(file_name(&key("a/b", WHEN), true).is_none());
    }

    #[test]
    fn active_entries_preserve_anomalies() {
        // Range and known-flag checks prevent false positives while carving slack, but active
        // entries have a trustworthy structure and must retain anomalous evidence.
        let mut unusual = key("timestomped.txt", 1);
        unusual[0x38..0x3c].copy_from_slice(&0x0080_0020u32.to_le_bytes());
        let live = [
            entry(64, &unusual, 0),
            entry(65, &key("after.txt", WHEN), 0),
        ];
        let r = &index_records(&record(&live, &[], false))[0];
        assert_eq!(r.entries.len(), 2);
        assert_eq!(r.entries[0].created, 1);
        assert_eq!(r.entries[0].file_flags.bits(), 0x0080_0020);
        assert!(file_name(&unusual, true).is_none());
    }

    #[test]
    fn torn_sector_keeps_the_bytes_found_in_evidence() {
        // The first sector ends inside this long active name. Its USA holds "a", while the
        // sector contains "b": a torn record must retain "b" rather than silently restoring the
        // stale USA byte pair.
        let long = "a".repeat(200);
        let mut data = record(&[entry(64, &key(&long, WHEN), 0)], &[], false);
        data[SECTOR - 2..SECTOR].copy_from_slice(&[b'b', 0]);
        let r = &index_records(&data)[0];
        assert!(!r.valid_fixup);
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].name.chars().nth(170), Some('b'));
    }

    #[test]
    fn copy_detection_does_not_cross_directories() {
        // The same file metadata can be left in directory 5 after a move to directory 6. That
        // old name is evidence, not an index-rebalancing copy of the live entry in directory 6.
        let live = [entry(64, &key_under("moved.txt", WHEN, 6), 0)];
        let slack = entry(64, &key_under("moved.txt", WHEN, 5), 0);
        let r = &index_records(&record(&live, &slack, false))[0];
        let moved = r.slack.iter().find(|e| e.name == "moved.txt").unwrap();
        assert!(!moved.copy_of_entry_in_use);
    }

    #[test]
    fn rejects_invalid_entry_and_fixup_layouts() {
        // A subnode entry needs a separate eight-byte VCN after its key.
        let e = entry(64, &key("no-vcn.txt", WHEN), ENTRY_SUBNODE);
        assert!(entry_at(&e, 0, e.len(), false).is_none());

        // The update sequence array must fit before the first sector's protected tail.
        let mut data = record(&[], &[], false);
        data[4..6].copy_from_slice(&500u16.to_le_bytes());
        assert!(index_records(&data).is_empty());
    }
}
