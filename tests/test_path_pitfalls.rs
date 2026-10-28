//! Full paths against the pitfalls of resolving paths from the MFT that Harel Segev lists in
//! "Resolving File Paths Using the MFT" (https://harelsegev.github.io/posts/resolving-file-paths-using-the-mft/):
//! reused and deleted parents (sequence numbers), short names and hard links, names that only an
//! extension record holds, files without a `$FILE_NAME`, and the extension records a reused parent
//! leaves behind. Each case is a small MFT built here, record by record.

use mft::MftParser;
use std::path::PathBuf;

const RECORD_SIZE: usize = 1024;
const RECORDS: usize = 64;
const ROOT: (u64, u16) = (5, 5);

const POSIX: u8 = 0;
const WIN32: u8 = 1;
const DOS: u8 = 2;

/// A `FILE` record to place in the test MFT.
struct Record {
    number: u64,
    sequence: u16,
    allocated: bool,
    directory: bool,
    /// The base record of an extension record; `(0, 0)` for a base record.
    base: (u64, u16),
    attributes: Vec<Vec<u8>>,
}

impl Record {
    fn new(number: u64, sequence: u16) -> Self {
        Record {
            number,
            sequence,
            allocated: true,
            directory: false,
            base: (0, 0),
            attributes: Vec::new(),
        }
    }
    fn dir(mut self) -> Self {
        self.directory = true;
        self
    }
    fn free(mut self) -> Self {
        self.allocated = false;
        self
    }
    fn extension_of(mut self, base: (u64, u16)) -> Self {
        self.base = base;
        self
    }
    fn with(mut self, attribute: Vec<u8>) -> Self {
        self.attributes.push(attribute);
        self
    }
    fn named(self, name: &str, namespace: u8, parent: (u64, u16)) -> Self {
        let directory = self.directory;
        self.with(file_name(name, namespace, parent, directory))
    }
}

fn reference((entry, sequence): (u64, u16)) -> [u8; 8] {
    let mut r = entry.to_le_bytes();
    r[6..8].copy_from_slice(&sequence.to_le_bytes());
    r
}

/// A resident attribute record with `value`.
fn resident(type_code: u32, value: &[u8]) -> Vec<u8> {
    let length = (0x18 + value.len() + 7) & !7;
    let mut a = vec![0u8; length];
    a[0..4].copy_from_slice(&type_code.to_le_bytes());
    a[4..8].copy_from_slice(&(length as u32).to_le_bytes());
    a[0x0A..0x0C].copy_from_slice(&0x18u16.to_le_bytes()); // name offset
    a[0x10..0x14].copy_from_slice(&(value.len() as u32).to_le_bytes());
    a[0x14..0x16].copy_from_slice(&0x18u16.to_le_bytes()); // value offset
    a[0x18..0x18 + value.len()].copy_from_slice(value);
    a
}

/// A `$FILE_NAME` attribute.
fn file_name(name: &str, namespace: u8, parent: (u64, u16), directory: bool) -> Vec<u8> {
    const FILETIME: [u8; 8] = [0xD5, 0x2D, 0x48, 0x58, 0x43, 0x5F, 0xCE, 0x01];
    let mut v = Vec::new();
    v.extend_from_slice(&reference(parent));
    for _ in 0..4 {
        v.extend_from_slice(&FILETIME);
    }
    v.extend_from_slice(&[0; 16]); // allocated and real size
    let flags: u32 = if directory { 0x1000_0000 } else { 0x20 };
    v.extend_from_slice(&flags.to_le_bytes());
    v.extend_from_slice(&[0; 4]); // reparse value
    let utf16: Vec<u16> = name.encode_utf16().collect();
    v.push(utf16.len() as u8);
    v.push(namespace);
    for c in utf16 {
        v.extend_from_slice(&c.to_le_bytes());
    }
    resident(0x30, &v)
}

/// A resident `$ATTRIBUTE_LIST` naming the record that holds each attribute.
fn attribute_list(entries: &[(u32, (u64, u16))]) -> Vec<u8> {
    let mut v = Vec::new();
    for (i, (type_code, segment)) in entries.iter().enumerate() {
        let mut e = [0u8; 0x20];
        e[0..4].copy_from_slice(&type_code.to_le_bytes());
        e[4..6].copy_from_slice(&0x20u16.to_le_bytes()); // entry length
        e[7] = 0x1A; // name offset
        e[0x10..0x18].copy_from_slice(&reference(*segment));
        e[0x18..0x1A].copy_from_slice(&(i as u16).to_le_bytes()); // attribute id
        v.extend_from_slice(&e);
    }
    resident(0x20, &v)
}

/// A non-resident `$ATTRIBUTE_LIST`: its clusters aren't in the MFT, so the records it names
/// can't be read from it.
fn non_resident_attribute_list() -> Vec<u8> {
    let mut a = vec![0u8; 0x48];
    a[0..4].copy_from_slice(&0x20u32.to_le_bytes());
    a[4..8].copy_from_slice(&0x48u32.to_le_bytes());
    a[8] = 1; // non-resident
    a[0x0A..0x0C].copy_from_slice(&0x40u16.to_le_bytes()); // name offset
    a[0x20..0x22].copy_from_slice(&0x40u16.to_le_bytes()); // data runs offset
    a[0x28..0x30].copy_from_slice(&4096u64.to_le_bytes()); // allocated size
    a[0x30..0x38].copy_from_slice(&0x100u64.to_le_bytes()); // real size
    a[0x38..0x40].copy_from_slice(&0x100u64.to_le_bytes()); // initialized size
    a[0x40..0x44].copy_from_slice(&[0x11, 0x01, 0x40, 0x00]); // 1 cluster at LCN 64
    a
}

fn write_record(mft: &mut [u8], r: &Record) {
    let at = r.number as usize * RECORD_SIZE;
    let e = &mut mft[at..at + RECORD_SIZE];
    e.fill(0);
    e[0..4].copy_from_slice(b"FILE");
    e[4..6].copy_from_slice(&0x30u16.to_le_bytes()); // update sequence array offset
    e[6..8].copy_from_slice(&3u16.to_le_bytes()); // update sequence array size
    e[0x10..0x12].copy_from_slice(&r.sequence.to_le_bytes());
    e[0x12..0x14].copy_from_slice(&1u16.to_le_bytes()); // hard link count
    e[0x14..0x16].copy_from_slice(&0x38u16.to_le_bytes()); // first attribute
    let flags = u16::from(r.allocated) | if r.directory { 2 } else { 0 };
    e[0x16..0x18].copy_from_slice(&flags.to_le_bytes());
    e[0x1C..0x20].copy_from_slice(&(RECORD_SIZE as u32).to_le_bytes());
    e[0x20..0x28].copy_from_slice(&reference(r.base));
    e[0x2C..0x30].copy_from_slice(&(r.number as u32).to_le_bytes());
    let mut offset = 0x38;
    for a in &r.attributes {
        e[offset..offset + a.len()].copy_from_slice(a);
        offset += a.len();
    }
    e[offset..offset + 4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    e[0x18..0x1C].copy_from_slice(&((offset + 8) as u32).to_le_bytes()); // used size
    // Update sequence number 1: the original sector ends (zero) are in the array, which is zero.
    assert!(
        offset + 8 <= 510,
        "the test record must not reach a sector end"
    );
    e[0x30..0x32].copy_from_slice(&1u16.to_le_bytes());
    e[510..512].copy_from_slice(&1u16.to_le_bytes());
    e[1022..1024].copy_from_slice(&1u16.to_le_bytes());
}

/// An MFT of `RECORDS` records: the given ones, the root directory (record 5), and free, empty
/// records everywhere else.
fn build(records: &[Record]) -> MftParser<std::io::Cursor<Vec<u8>>> {
    let mut mft = vec![0u8; RECORDS * RECORD_SIZE];
    for number in 0..RECORDS as u64 {
        write_record(&mut mft, &Record::new(number, 1).free());
    }
    write_record(&mut mft, &Record::new(5, 5).dir().named(".", WIN32, ROOT));
    for r in records {
        write_record(&mut mft, r);
    }
    MftParser::from_buffer(mft).unwrap()
}

fn path_of(parser: &mut MftParser<std::io::Cursor<Vec<u8>>>, number: u64) -> Option<PathBuf> {
    let entry = parser.get_entry(number).unwrap();
    parser.get_full_path_for_entry(&entry).unwrap()
}

fn path(components: &str) -> Option<PathBuf> {
    Some(components.split('/').collect())
}

/// `Users` (20) and `Users/alice` (21), the parents of the cases below.
fn users() -> Vec<Record> {
    vec![
        Record::new(20, 1).dir().named("Users", WIN32, ROOT),
        Record::new(21, 1).dir().named("alice", WIN32, (20, 1)),
    ]
}

#[test]
fn a_live_tree_resolves() {
    let mut records = users();
    records.push(Record::new(22, 1).named("notes.txt", WIN32, (21, 1)));
    let mut parser = build(&records);
    assert_eq!(path_of(&mut parser, 21), path("Users/alice"));
    assert_eq!(path_of(&mut parser, 22), path("Users/alice/notes.txt"));
}

/// Pitfall 1: the parent's record was freed and reused by another file, so the reference's
/// sequence number no longer matches. The file is an orphan, not a child of the new file.
#[test]
fn a_reused_parent_makes_its_children_orphans() {
    let mut records = users();
    // Record 23 was the directory `old` (sequence 1), freed (2) and reused as a directory again.
    records.push(Record::new(23, 2).dir().named("new", WIN32, (20, 1)));
    records.push(Record::new(24, 2).free().named("lost.txt", WIN32, (23, 1)));
    // The current version's children still resolve.
    records.push(Record::new(25, 1).named("kept.txt", WIN32, (23, 2)));
    let mut parser = build(&records);
    assert_eq!(path_of(&mut parser, 24), path("[Orphaned]/lost.txt"));
    assert_eq!(path_of(&mut parser, 25), path("Users/new/kept.txt"));
}

/// Pitfall 1: a deleted directory that wasn't reused is still the parent of its deleted files;
/// freeing it raised its sequence number by one.
#[test]
fn a_deleted_parent_keeps_its_children() {
    let mut records = users();
    records.push(
        Record::new(26, 2)
            .dir()
            .free()
            .named("Temp", WIN32, (20, 1)),
    );
    records.push(Record::new(27, 2).free().named("a.tmp", WIN32, (26, 1)));
    let mut parser = build(&records);
    assert_eq!(path_of(&mut parser, 27), path("Users/Temp/a.tmp"));
}

/// Pitfall 2: a file with a long name and a short name (two `$FILE_NAME`s) is shown by its long
/// name, whatever the attributes' order.
#[test]
fn short_names_give_way_to_long_names() {
    let mut records = users();
    records.push(
        Record::new(28, 1)
            .named("LONGFI~1.TXT", DOS, (21, 1))
            .named("Long File Name.txt", WIN32, (21, 1)),
    );
    records.push(
        Record::new(29, 1)
            .named("posix name", POSIX, (21, 1))
            .named("POSIXN~1", DOS, (21, 1)),
    );
    let mut parser = build(&records);
    assert_eq!(
        path_of(&mut parser, 28),
        path("Users/alice/Long File Name.txt")
    );
    assert_eq!(path_of(&mut parser, 29), path("Users/alice/posix name"));
}

/// Pitfall 3: a directory's base record only has room for its 8.3 name, and its long name is in
/// an extension record, named by a resident `$ATTRIBUTE_LIST`.
#[test]
fn a_long_name_in_an_extension_record_is_used() {
    let records = vec![
        Record::new(30, 1)
            .dir()
            .named("AMD64_~1", DOS, ROOT)
            .with(attribute_list(&[(0x30, (30, 1)), (0x30, (31, 1))])),
        Record::new(31, 1).dir().extension_of((30, 1)).named(
            "amd64_microsoft-windows-component",
            WIN32,
            ROOT,
        ),
        Record::new(32, 1).named("x.dll", WIN32, (30, 1)),
    ];
    let mut parser = build(&records);
    assert_eq!(
        path_of(&mut parser, 32),
        path("amd64_microsoft-windows-component/x.dll")
    );
}

/// Pitfall 3: the `$ATTRIBUTE_LIST` is non-resident, so the extension record is found by its
/// base reference instead.
#[test]
fn a_long_name_behind_a_non_resident_attribute_list_is_used() {
    let records = vec![
        Record::new(33, 1)
            .dir()
            .named("LONGNA~1", DOS, ROOT)
            .with(non_resident_attribute_list()),
        Record::new(34, 1).dir().extension_of((33, 1)).named(
            "Long name found by a scan",
            WIN32,
            ROOT,
        ),
        Record::new(35, 1).named("y.txt", WIN32, (33, 1)),
    ];
    let mut parser = build(&records);
    assert_eq!(
        path_of(&mut parser, 35),
        path("Long name found by a scan/y.txt")
    );
}

/// Pitfall 3: when only a directory's short name is left, it is used.
#[test]
fn a_short_name_is_used_when_it_is_all_there_is() {
    let records = vec![
        Record::new(36, 1).dir().named("PROGRA~1", DOS, ROOT),
        Record::new(37, 1).named("z.txt", WIN32, (36, 1)),
    ];
    let mut parser = build(&records);
    assert_eq!(path_of(&mut parser, 37), path("PROGRA~1/z.txt"));
}

/// Pitfall 3: a file without a `$FILE_NAME` (its record partly reused) has no path, rather than
/// being called an orphan: its parent may well still exist.
#[test]
fn a_file_without_a_name_has_no_path() {
    let mut parser = build(&[Record::new(38, 1)]);
    assert_eq!(path_of(&mut parser, 38), None);
}

/// Pitfall 3 ("orphaned attributes"): the parent directory was deleted and its base record reused
/// by another file, but its extension record, which held its long name, is still there and still
/// points back at the old version. The parent's name and its own parent come from there.
#[test]
fn a_reused_parents_old_extension_record_still_names_it() {
    let mut records = users();
    records.extend([
        // Record 40 was the directory `AMD64_~1` (sequence 1, long name in record 41); deleted
        // (2) and reused by a file.
        Record::new(40, 2).named("reuser.bin", WIN32, ROOT),
        Record::new(41, 2).dir().free().extension_of((40, 1)).named(
            "amd64_old_component",
            WIN32,
            (20, 1),
        ),
        Record::new(42, 2)
            .free()
            .named("old.manifest", WIN32, (40, 1)),
    ]);
    let mut parser = build(&records);
    assert_eq!(
        path_of(&mut parser, 42),
        path("Users/amd64_old_component/old.manifest")
    );
    // The file that reused the record is unaffected.
    assert_eq!(path_of(&mut parser, 40), path("reuser.bin"));
}

/// Two directories that name each other as parent, and a directory that names itself: the walk
/// ends, cutting the cycle at its lowest record.
#[test]
fn parent_cycles_are_cut() {
    let records = vec![
        Record::new(43, 1).dir().named("loopA", WIN32, (44, 1)),
        Record::new(44, 1).dir().named("loopB", WIN32, (43, 1)),
        Record::new(45, 1).named("in-loop.txt", WIN32, (44, 1)),
        Record::new(46, 1).dir().named("self", WIN32, (46, 1)),
    ];
    let mut parser = build(&records);
    assert_eq!(
        path_of(&mut parser, 45),
        path("[Orphaned]/loopA/loopB/in-loop.txt")
    );
    assert_eq!(path_of(&mut parser, 46), path("[Orphaned]/self"));
}

/// A parent that isn't a directory, or isn't in the MFT, can't be trusted: `[Unknown]`.
#[test]
fn an_unusable_parent_is_unknown() {
    let records = vec![
        Record::new(47, 1).named("not-a-directory", WIN32, ROOT),
        Record::new(48, 1).named("child", WIN32, (47, 1)),
        Record::new(49, 1).named("far", WIN32, (9999, 1)),
    ];
    let mut parser = build(&records);
    assert_eq!(path_of(&mut parser, 48), path("[Unknown]/child"));
    assert_eq!(path_of(&mut parser, 49), path("[Unknown]/far"));
}

/// Leftover extension records have no base record among them, so the best name wins across all of
/// them (Win32 over POSIX over DOS), whatever their order.
#[test]
fn leftover_names_are_ranked_across_all_records() {
    for (posix, win32) in [(51, 52), (52, 51)] {
        let mut records = users();
        records.extend([
            Record::new(50, 2).named("reuser.bin", WIN32, ROOT),
            Record::new(posix, 2)
                .dir()
                .free()
                .extension_of((50, 1))
                .named("posix-old", POSIX, (20, 1)),
            Record::new(win32, 2)
                .dir()
                .free()
                .extension_of((50, 1))
                .named("Win32 Old", WIN32, (20, 1)),
            Record::new(53, 2).free().named("child.txt", WIN32, (50, 1)),
        ]);
        let mut parser = build(&records);
        assert_eq!(
            path_of(&mut parser, 53),
            path("Users/Win32 Old/child.txt"),
            "POSIX name in record {posix}"
        );
    }
    // A short name is used only when it is all there is.
    let mut records = users();
    records.extend([
        Record::new(50, 2).named("reuser.bin", WIN32, ROOT),
        Record::new(51, 2)
            .dir()
            .free()
            .extension_of((50, 1))
            .named("OLDDIR~1", DOS, (20, 1)),
        Record::new(52, 2)
            .dir()
            .free()
            .extension_of((50, 1))
            .named("old dir", POSIX, (20, 1)),
        Record::new(53, 2).free().named("child.txt", WIN32, (50, 1)),
    ]);
    let mut parser = build(&records);
    assert_eq!(path_of(&mut parser, 53), path("Users/old dir/child.txt"));
}
