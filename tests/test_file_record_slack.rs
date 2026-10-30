//! File names left in the slack of FILE records (past the size in use): a `$FILE_NAME` attribute
//! and an index entry the record had before, found across a sector end (the update sequence array
//! applied) and with their offsets in the record. Each record is built here.

use mft::MftParser;

const RECORD_SIZE: usize = 1024;
const SECTOR: usize = 512;
/// 2013-06-02T18:04:04Z, a time from 1980 to 2100.
const FILETIME: u64 = 0x01CE_5F43_5848_2DD5;

fn reference(entry: u64, sequence: u16) -> [u8; 8] {
    (entry | u64::from(sequence) << 48).to_le_bytes()
}

/// A `$FILE_NAME` value for `name` under directory `parent`.
fn file_name(name: &str, parent: (u64, u16)) -> Vec<u8> {
    let mut v = reference(parent.0, parent.1).to_vec();
    for _ in 0..4 {
        v.extend_from_slice(&FILETIME.to_le_bytes());
    }
    v.extend_from_slice(&4096u64.to_le_bytes());
    v.extend_from_slice(&1234u64.to_le_bytes());
    v.extend_from_slice(&0x20u32.to_le_bytes());
    v.extend_from_slice(&[0; 4]);
    let utf16: Vec<u16> = name.encode_utf16().collect();
    v.push(utf16.len() as u8);
    v.push(1); // Win32
    v.extend(utf16.iter().flat_map(|c| c.to_le_bytes()));
    v
}

/// A resident attribute record with `value`.
fn resident(type_code: u32, value: &[u8]) -> Vec<u8> {
    let length = (0x18 + value.len()).next_multiple_of(8);
    let mut a = vec![0u8; length];
    a[0..4].copy_from_slice(&type_code.to_le_bytes());
    a[4..8].copy_from_slice(&(length as u32).to_le_bytes());
    a[0x0A..0x0C].copy_from_slice(&0x18u16.to_le_bytes());
    a[0x10..0x14].copy_from_slice(&(value.len() as u32).to_le_bytes());
    a[0x14..0x16].copy_from_slice(&0x18u16.to_le_bytes());
    a[0x18..0x18 + value.len()].copy_from_slice(value);
    a
}

/// An index entry for `file` keyed by `key`.
fn index_entry(file: (u64, u16), key: &[u8]) -> Vec<u8> {
    let mut e = reference(file.0, file.1).to_vec();
    let length = (0x10 + key.len()).next_multiple_of(8);
    e.extend_from_slice(&(length as u16).to_le_bytes());
    e.extend_from_slice(&(key.len() as u16).to_le_bytes());
    e.extend_from_slice(&[0; 4]);
    e.extend_from_slice(key);
    e.resize(length, 0);
    e
}

/// FILE record 40 (sequence 3) with `attributes`, its end marker, then `slack` at `slack_at`;
/// protected by an update sequence array (value 7) like a record on disk.
fn record(attributes: &[Vec<u8>], slack_at: usize, slack: &[u8]) -> Vec<u8> {
    let mut e = vec![0u8; RECORD_SIZE];
    e[0..4].copy_from_slice(b"FILE");
    e[4..6].copy_from_slice(&0x30u16.to_le_bytes());
    e[6..8].copy_from_slice(&3u16.to_le_bytes());
    e[0x10..0x12].copy_from_slice(&3u16.to_le_bytes());
    e[0x12..0x14].copy_from_slice(&1u16.to_le_bytes());
    e[0x14..0x16].copy_from_slice(&0x38u16.to_le_bytes());
    e[0x16..0x18].copy_from_slice(&1u16.to_le_bytes());
    e[0x1C..0x20].copy_from_slice(&(RECORD_SIZE as u32).to_le_bytes());
    e[0x2C..0x30].copy_from_slice(&40u32.to_le_bytes());
    let mut at = 0x38;
    for a in attributes {
        e[at..at + a.len()].copy_from_slice(a);
        at += a.len();
    }
    e[at..at + 4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    e[0x18..0x1C].copy_from_slice(&((at + 8) as u32).to_le_bytes());
    assert!(slack_at >= at + 8);
    e[slack_at..slack_at + slack.len()].copy_from_slice(slack);
    // The update sequence array: value 7, then each sector's last two bytes.
    e[0x30..0x32].copy_from_slice(&7u16.to_le_bytes());
    for sector in 0..RECORD_SIZE / SECTOR {
        let end = (sector + 1) * SECTOR - 2;
        let kept = [e[end], e[end + 1]];
        e[0x32 + 2 * sector..0x34 + 2 * sector].copy_from_slice(&kept);
        e[end..end + 2].copy_from_slice(&7u16.to_le_bytes());
    }
    e
}

fn mft_with(record: Vec<u8>) -> MftParser<std::io::Cursor<Vec<u8>>> {
    let mut mft = vec![0u8; 41 * RECORD_SIZE];
    mft[40 * RECORD_SIZE..].copy_from_slice(&record);
    MftParser::from_buffer(mft).unwrap()
}

#[test]
fn names_left_in_the_slack_of_a_file_record() {
    let current = resident(0x30, &file_name("report.txt", (5, 5)));
    // An earlier name, as a whole attribute, across the first sector's end; and an entry of an
    // index root that shrank.
    let earlier = resident(0x30, &file_name("invoice.pdf.exe", (90, 2)));
    let mut slack = earlier.clone();
    slack.extend(index_entry((77, 4), &file_name("notes.lnk", (40, 3))));
    let slack_at = 0x1C8;
    let used = 0x38 + current.len() + 8;
    let mut parser = mft_with(record(&[current], slack_at, &slack));
    let entry = parser.get_entry(40).unwrap();
    assert_eq!(entry.valid_fixup, Some(true));
    let (start, bytes) = entry.slack();
    assert_eq!(start, used);
    assert_eq!(bytes.len(), RECORD_SIZE - start);

    let names = entry.slack_file_names();
    assert_eq!(names.len(), 2, "{names:#?}");
    let earlier_name = &names[0];
    assert_eq!(earlier_name.name, "invoice.pdf.exe");
    assert_eq!(earlier_name.offset, (slack_at + 0x18) as u64);
    assert_eq!(earlier_name.file_reference, None);
    let parent = earlier_name.parent.unwrap();
    assert_eq!((parent.entry, parent.sequence), (90, 2));
    assert_eq!(earlier_name.real_size, 1234);
    assert!(earlier_name.from_slack);

    let entry_name = &names[1];
    assert_eq!(entry_name.name, "notes.lnk");
    assert_eq!(entry_name.offset, (slack_at + earlier.len()) as u64);
    let file = entry_name.file_reference.unwrap();
    assert_eq!((file.entry, file.sequence), (77, 4));
}

#[test]
fn no_names_in_empty_or_unreadable_slack() {
    let current = resident(0x30, &file_name("report.txt", (5, 5)));
    let mut parser = mft_with(record(std::slice::from_ref(&current), 0x200, &[]));
    assert_eq!(parser.get_entry(40).unwrap().slack_file_names(), []);

    // Not names: random-looking bytes.
    let noise: Vec<u8> = (0..400u32).map(|i| (i * 37 % 251) as u8).collect();
    let mut parser = mft_with(record(std::slice::from_ref(&current), 0x100, &noise));
    assert_eq!(parser.get_entry(40).unwrap().slack_file_names(), []);

    // A used size past the record: no slack.
    let mut damaged = record(&[current], 0x200, &[]);
    damaged[0x18..0x1C].copy_from_slice(&5000u32.to_le_bytes());
    let mut parser = mft_with(damaged);
    let entry = parser.get_entry(40).unwrap();
    assert_eq!(entry.slack().1, []);
    assert_eq!(entry.slack_file_names(), []);
}
