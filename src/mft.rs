use crate::entry::MftEntry;
use crate::err::{Error, Result};

use crate::EntryHeader;
use log::{debug, trace};

use lru::LruCache;
use std::fs::{self, File};
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::num::NonZeroUsize;
use std::path::Path;

pub struct MftParser<T: Read + Seek> {
    data: T,
    /// Entry size is present in the volume header, but this is not available to us.
    /// Instead this will be guessed by the entry size of the first entry.
    entry_size: u32,
    size: u64,
    entries_cache: LruCache<(u64, u16), String>,
    // Next expected read offset in the underlying stream. Used to avoid a seek syscall when
    // reading entries sequentially.
    next_read_offset: u64,
}

impl MftParser<BufReader<File>> {
    /// Instantiates an instance of the parser from a file path.
    /// Does not mutate the file contents in any way.
    pub fn from_path(filename: impl AsRef<Path>) -> Result<Self> {
        let f = filename.as_ref();

        let mft_fh = File::open(f).map_err(|e| Error::failed_to_open_file(f, e))?;
        let size = fs::metadata(f)?.len();

        Self::from_read_seek(BufReader::with_capacity(4096, mft_fh), Some(size))
    }
}

impl MftParser<Cursor<Vec<u8>>> {
    /// Instantiates an instance of the parser from a buffer containing a full MFT file.
    /// Useful for testing.
    pub fn from_buffer(buffer: Vec<u8>) -> Result<Self> {
        let size = buffer.len() as u64;
        let cursor = Cursor::new(buffer);

        Self::from_read_seek(cursor, Some(size))
    }
}

/// Joins an MFT path and a name with `/`, the same on every OS (an empty parent is the root).
fn join_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// Parent references followed at most for one path. Real paths are far shallower; the limit keeps a
/// crafted chain of nested directories from exhausting the stack.
const MAX_PATH_DEPTH: usize = 256;

/// The largest MFT entry size accepted (NTFS uses 1024 or 4096 bytes). A bigger size in a damaged
/// or crafted header would make every entry read allocate that much.
const MAX_ENTRY_SIZE: u32 = 64 * 1024;
/// How many entry positions are tried to find a usable header (the first 4 MiB at 1 KiB steps).
const MAX_ENTRY_SIZE_PROBES: usize = 4096;

fn check_entry_size<T: Read + Seek>(data: &mut T, off: u64) -> Result<u32> {
    data.seek(SeekFrom::Start(off))?;
    if let Ok(header) = EntryHeader::from_reader(data, 0) {
        // Check that the entry size is nonzero and is consistent with
        // the length of the update sequence array. There should be one
        // element of the USA for each 512 bytes of entry, plus one extra
        // element for the update sequence number. The probability that
        // the entry size and USA size follow this rule when selected
        // randomly is vanishingly small (1/2^32), so this check should be
        // sufficient to ensure that we have a good entry size.
        let size = header.total_entry_size;
        let usa_sectors = header.usa_size.checked_sub(1).map(u32::from);
        if size.is_power_of_two()
            && (512..=MAX_ENTRY_SIZE).contains(&size)
            && usa_sectors.is_some_and(|n| n * 512 == size)
        {
            return Ok(size);
        }
    }
    Err(Error::FailedToReadEntrySize {})
}

/// Search for a valid MFT FILE record to determine the FILE record size.
fn find_entry_size<T: Read + Seek>(data: &mut T, size: u64) -> Result<u32> {
    // Each valid FILE entry header contains the total size of its entry.
    // Every entry in the MFT is the same size, so we get the entry size
    // for this MFT from the first good FILE entry we can find. The stride
    // for the search is 1024; in the event that the entry size is 4096,
    // a stride of 1024 still works, due to 4096 being a multiple of 1024.
    //
    // Corrupt MFTs may have the first entries wiped/overwritten, and using
    // this method of determining the FILE record size may allow for recovery
    // of entries in such cases. The search is bounded so that a large file
    // that is not an MFT is not read to its end.
    (0..size)
        .step_by(1024)
        .take(MAX_ENTRY_SIZE_PROBES)
        .find_map(|off| check_entry_size(data, off).ok())
        .ok_or(Error::FailedToReadEntrySize {})
}

impl<T: Read + Seek> MftParser<T> {
    pub fn from_read_seek(mut data: T, size: Option<u64>) -> Result<Self> {
        let size = match size {
            Some(sz) => sz,
            None => data.seek(SeekFrom::End(0))?,
        };

        let entry_size = find_entry_size(&mut data, size)?;
        data.seek(SeekFrom::Start(0))?;

        Ok(Self {
            data,
            entry_size,
            size,
            entries_cache: LruCache::new(NonZeroUsize::new(1000).expect("1000 > 0")),
            next_read_offset: 0,
        })
    }

    pub fn get_entry_count(&self) -> u64 {
        self.size / u64::from(self.entry_size)
    }

    /// Reads an entry from the MFT by entry number.
    pub fn get_entry(&mut self, entry_number: u64) -> Result<MftEntry> {
        debug!("Reading entry {entry_number}");

        let entry_size = u64::from(self.entry_size);
        let desired_offset = entry_number
            .checked_mul(entry_size)
            .ok_or_else(|| Error::Any {
                detail: format!("entry number {entry_number} is beyond the MFT"),
            })?;

        // Avoid seeking when reading sequential entries; this keeps the BufReader hot and
        // removes a large number of __lseek syscalls in tight loops.
        if self.next_read_offset != desired_offset {
            self.data.seek(SeekFrom::Start(desired_offset))?;
            self.next_read_offset = desired_offset;
        }

        let mut entry_buffer = vec![0; self.entry_size as usize];

        self.data.read_exact(&mut entry_buffer)?;
        self.next_read_offset = self.next_read_offset.saturating_add(entry_size);

        MftEntry::from_buffer(entry_buffer, entry_number)
    }

    /// Iterates over all the entries in the MFT.
    pub fn iter_entries(&mut self) -> impl Iterator<Item = Result<MftEntry>> + '_ {
        let total_entries = self.get_entry_count();

        (0..total_entries).map(move |i| self.get_entry(i))
    }

    #[cfg(test)]
    fn inner_get_entry(
        &mut self,
        parent_reference: (u64, u16),
        entry_name: Option<&str>,
    ) -> String {
        self.resolve_parent(parent_reference, entry_name, &mut Vec::new())
    }

    /// The path of the parent a reference points to, joined with `entry_name`. `chain` holds the
    /// entries being resolved, so that a parent chain that loops back on itself (or never ends) in
    /// a damaged or crafted MFT ends in `[Unknown]` instead of recursing until the stack overflows.
    fn resolve_parent(
        &mut self,
        parent_reference: (u64, u16),
        entry_name: Option<&str>,
        chain: &mut Vec<u64>,
    ) -> String {
        // If my parent path is known, then my path is parent's full path + my name.
        // Else, retrieve and cache my parent's path.
        let path = if let Some(cached_parent_path) = self.entries_cache.get(&parent_reference) {
            cached_parent_path.clone()
        } else {
            let (parent_entry_id, parent_sequence) = parent_reference;
            if chain.contains(&parent_entry_id) || chain.len() >= MAX_PATH_DEPTH {
                trace!(
                    "Found a parent reference loop or too deep a path at entry ID {parent_entry_id}"
                );
                String::from("[Unknown]")
            } else {
                chain.push(parent_entry_id);
                let path = match self.get_entry(parent_entry_id).ok() {
                    Some(parent) if !is_referenced_version(&parent, parent_sequence) => {
                        trace!(
                            "Found stale MFT reference: entry ID {parent_entry_id}, \
                             expected sequence {parent_sequence}, actual sequence {}",
                            parent.header.sequence
                        );
                        String::from("[Orphaned]")
                    }
                    Some(parent) => match self.path_of(&parent, chain) {
                        Some(path) if parent.is_dir() => path,
                        Some(_) => String::from("[Unknown]"),
                        // I have a parent, which doesn't have a filename attribute.
                        // Default to root.
                        None => String::new(),
                    },
                    // Parent is maybe corrupted or incomplete, use a sentinel instead.
                    None => String::from("[Unknown]"),
                };
                chain.pop();
                self.entries_cache.put(parent_reference, path.clone());
                path
            }
        };
        match entry_name {
            Some(name) => join_path(&path, name),
            None => path,
        }
    }

    /// Gets the full path for an entry, with `/` separators on every OS.
    /// Caches computations.
    pub fn get_full_path_for_entry(&mut self, entry: &MftEntry) -> Result<Option<String>> {
        Ok(self.path_of(entry, &mut Vec::new()))
    }

    fn path_of(&mut self, entry: &MftEntry, chain: &mut Vec<u64>) -> Option<String> {
        let entry_id = entry.header.record_number;
        match entry.find_best_name_attribute() {
            Some(filename_header) => {
                let parent_reference = (
                    filename_header.parent.entry,
                    filename_header.parent.sequence,
                );
                let parent_entry_id = parent_reference.0;
                let name = filename_header.name.to_utf8_string();

                // MFT entry 5 is the root path.
                if parent_entry_id == 5 {
                    return Some(name);
                }

                if parent_entry_id == entry_id {
                    trace!("Found self-referential file path, for entry ID {entry_id}");
                    return Some(join_path("[Orphaned]", &name));
                }

                if parent_entry_id > 0 {
                    Some(self.resolve_parent(parent_reference, Some(&name), chain))
                } else {
                    trace!("Found orphaned entry ID {entry_id}");

                    let orphan = join_path("[Orphaned]", &name);

                    self.entries_cache.put(
                        (entry.header.record_number, entry.header.sequence),
                        orphan.clone(),
                    );

                    Some(orphan)
                }
            }
            None => match (
                entry.header.base_reference.entry,
                entry.header.base_reference.sequence,
            ) {
                // I don't have a parent reference, and no X30 attribute. Though luck.
                (0, _) => None,
                parent_reference => Some(self.resolve_parent(parent_reference, None, chain)),
            },
        }
    }
}

/// Whether `entry` is the version of itself that a reference with `sequence` points to. NTFS
/// increments an entry's sequence number when the entry is freed, so the deleted parent directory
/// of a deleted file is one ahead of the file's reference; a sequence of 0 names no version.
fn is_referenced_version(entry: &MftEntry, sequence: u16) -> bool {
    let actual = entry.header.sequence;
    sequence == 0
        || actual == sequence
        || (!entry.is_allocated() && actual == sequence.wrapping_add(1))
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::{mft_sample, mft_sample_name};
    use crate::{MftEntry, MftParser};

    use std::fs::File;
    use std::io::{BufReader, Seek, SeekFrom};

    use super::{MAX_PATH_DEPTH, find_entry_size};

    // entrypoint for clion profiler.
    #[test]
    fn test_process_90_mft_entries() {
        let sample = mft_sample();

        let mut parser = MftParser::from_path(sample).unwrap();

        let mut count = 0;
        for record in parser.iter_entries().take(10000).filter_map(|a| a.ok()) {
            for _attribute in record.iter_attributes() {
                count += 1;
            }
        }

        assert!(count > 0)
    }

    #[test]
    fn test_get_full_path() {
        let sample = mft_sample();
        let mut parser = MftParser::from_path(sample).unwrap();

        let mut paths = Vec::with_capacity(1000);
        let entries: Vec<MftEntry> = parser
            .iter_entries()
            .take(1000)
            .filter_map(Result::ok)
            .collect();

        for entry in entries {
            if let Some(path) = parser.get_full_path_for_entry(&entry).unwrap() {
                paths.push(path)
            }
        }

        assert_eq!(paths.len(), 988);
    }

    #[test]
    fn test_get_full_name() {
        let sample = mft_sample();
        let mut parser = MftParser::from_path(sample).unwrap();

        let e = parser.get_entry(5).unwrap();
        parser.get_full_path_for_entry(&e).unwrap();

        // `/` on every OS (upstream PR #120)
        let e = parser.get_entry(500).unwrap();
        let path = parser.get_full_path_for_entry(&e).unwrap().unwrap();
        assert_eq!(path, "WINDOWS/system32/devmgmt.msc");
    }

    #[test]
    fn test_parent_sequence_mismatch_is_orphaned() {
        let sample = mft_sample();
        let mut parser = MftParser::from_path(sample).unwrap();

        // Entry 5 is the root and should have a valid sequence number.
        let root = parser.get_entry(5).unwrap();
        let root_sequence = root.header.sequence;

        // Reference entry 5 with an invalid/stale sequence number.
        let stale_sequence = root_sequence.wrapping_add(1);

        let path = parser.inner_get_entry((5, stale_sequence), Some("test.txt"));

        assert_eq!(path, "[Orphaned]/test.txt");
    }

    /// Point every $FILE_NAME of `entry` at `parent` (with the parent's current sequence number).
    fn set_parent(data: &mut [u8], entry: usize, parent: u64) {
        let size = 1024;
        let base = entry * size;
        let parent_sequence = {
            let p = parent as usize * size;
            u16::from_le_bytes([data[p + 0x10], data[p + 0x11]])
        };
        let mut off = usize::from(u16::from_le_bytes([data[base + 0x14], data[base + 0x15]]));
        while off + 8 < size {
            let t = u32::from_le_bytes(data[base + off..base + off + 4].try_into().unwrap());
            let len = u32::from_le_bytes(data[base + off + 4..base + off + 8].try_into().unwrap());
            if t == 0xFFFF_FFFF || len == 0 {
                break;
            }
            if t == 0x30 && data[base + off + 8] == 0 {
                let content = usize::from(u16::from_le_bytes([
                    data[base + off + 0x14],
                    data[base + off + 0x15],
                ]));
                let at = base + off + content;
                assert!(at % 512 + 8 <= 510, "must not touch a fixup position");
                data[at..at + 6].copy_from_slice(&parent.to_le_bytes()[..6]);
                data[at + 6..at + 8].copy_from_slice(&parent_sequence.to_le_bytes());
            }
            off += len as usize;
        }
    }

    /// Entries from 16 on that are directories with a $FILE_NAME.
    fn directories(data: &[u8]) -> Vec<usize> {
        (16..data.len() / 1024)
            .filter(|&i| {
                let base = i * 1024;
                &data[base..base + 4] == b"FILE" && data[base + 0x16] & 2 != 0
            })
            .collect()
    }

    #[test]
    fn test_deleted_parent_directory_keeps_its_path() {
        // Entry 12590 (deleted) references parent 12589 with sequence 1; the parent directory was
        // deleted too, which raised its sequence number to 2. It is still the same directory.
        let mut parser = MftParser::from_path(mft_sample()).unwrap();
        let parent = parser.get_entry(12589).unwrap();
        assert!(!parent.is_allocated());
        assert_eq!(parent.header.sequence, 2);
        let entry = parser.get_entry(12590).unwrap();
        assert_eq!(
            parser.get_full_path_for_entry(&entry).unwrap().unwrap(),
            "Documents and Settings/Donald Blake/Local Settings/History/History.IE5/MSHist012009011420090115/index.dat"
        );
        // a reference with sequence 0 names no version of the parent
        let root_path = parser.inner_get_entry((5, 5), Some("x"));
        assert_eq!(parser.inner_get_entry((5, 0), Some("x")), root_path);
    }

    #[test]
    fn test_parent_reference_loop_does_not_overflow_the_stack() {
        let mut data = std::fs::read(mft_sample()).unwrap();
        data.truncate(1024 * 1024);
        let dirs = directories(&data);
        let (a, b) = (dirs[0], dirs[1]);
        set_parent(&mut data, a, b as u64);
        set_parent(&mut data, b, a as u64);
        let mut parser = MftParser::from_buffer(data).unwrap();
        let entry = parser.get_entry(a as u64).unwrap();
        let path = parser.get_full_path_for_entry(&entry).unwrap().unwrap();
        assert!(path.starts_with("[Unknown]/"), "{path}");
        let paths = (0..parser.get_entry_count())
            .filter_map(|i| parser.get_entry(i).ok())
            .collect::<Vec<_>>();
        for e in &paths {
            parser.get_full_path_for_entry(e).unwrap();
        }
    }

    #[test]
    fn test_very_deep_parent_chains_stop_at_the_depth_limit() {
        let mut data = std::fs::read(mft_sample()).unwrap();
        let dirs = directories(&data);
        let chain = &dirs[..600];
        assert!(chain.len() > MAX_PATH_DEPTH);
        for pair in chain.windows(2) {
            set_parent(&mut data, pair[1], pair[0] as u64);
        }
        let mut parser = MftParser::from_buffer(data).unwrap();
        let deepest = parser.get_entry(*chain.last().unwrap() as u64).unwrap();
        let path = parser.get_full_path_for_entry(&deepest).unwrap().unwrap();
        assert!(path.starts_with("[Unknown]/"), "{}", &path[..40]);
        // [Unknown], the MAX_PATH_DEPTH resolved directories, then the entry's own name
        assert_eq!(path.matches('/').count(), MAX_PATH_DEPTH + 1);
    }

    #[test]
    fn test_huge_entry_numbers_are_errors() {
        let mut parser = MftParser::from_path(mft_sample()).unwrap();
        assert!(parser.get_entry(u64::MAX).is_err());
        assert!(parser.get_entry(parser.get_entry_count()).is_err());
        assert!(parser.get_entry(0).is_ok());
    }

    #[test]
    fn test_find_entry_size() {
        // The first header is zeroed. The second header has a corrupt
        // sequence array. The third header is valid.
        let f = File::open(mft_sample_name("third_header_good")).unwrap();
        let mut data = BufReader::new(f);
        let size = data.seek(SeekFrom::End(0)).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(find_entry_size(&mut data, size).unwrap(), 1024);
    }

    #[test]
    fn test_find_entry_size_rejects_unusable_headers() {
        let mut data = std::fs::read(mft_sample()).unwrap();
        data.truncate(4 * 1024);
        // entry 0: no update sequence array (usa_size 0 must not underflow)
        data[6..8].copy_from_slice(&0u16.to_le_bytes());
        // entry 1: a consistent but huge size (1 MiB, 2049 USA elements)
        data[1024 + 6..1024 + 8].copy_from_slice(&2049u16.to_le_bytes());
        data[1024 + 0x1C..1024 + 0x20].copy_from_slice(&(1024u32 * 1024).to_le_bytes());
        // entry 2: zeroed
        data[2048..3072].fill(0);
        // entry 3 is intact
        let size = data.len() as u64;
        let mut cursor = std::io::Cursor::new(data.clone());
        assert_eq!(find_entry_size(&mut cursor, size).unwrap(), 1024);
        data[3072..].fill(0);
        let mut cursor = std::io::Cursor::new(data.clone());
        assert!(find_entry_size(&mut cursor, size).is_err());
        assert!(MftParser::from_buffer(data).is_err());
        assert!(
            MftParser::from_buffer(vec![0; 4096]).is_err(),
            "no division by zero"
        );
    }
}
