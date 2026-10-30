//! Directory indexes (`$I30`) of a volume built with ntfs-3g (`samples/i30/`): the entries in
//! use, and the entries deleted or renamed files left in the slack. The same entries as dfir_ntfs
//! reads (in use and every slack occurrence).

use mft::i30::{IndexEntry, IndexRecord, index_records};
use std::panic::{self, AssertUnwindSafe};

fn sample(name: &str) -> Vec<u8> {
    let path = format!("{}/samples/i30/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(path).unwrap()
}

fn entries(records: &[IndexRecord], slack: bool) -> Vec<&IndexEntry> {
    records
        .iter()
        .flat_map(|r| if slack { &r.slack } else { &r.entries })
        .collect()
}

#[test]
fn logs_in_use_and_in_slack() {
    let records = index_records(&sample("Logs_I30"));
    assert_eq!(records.len(), 7);
    assert!(
        records
            .iter()
            .all(|r| r.valid_fixup && r.problems.is_empty())
    );
    let vcns: Vec<u64> = records.iter().map(|r| r.vcn).collect();
    assert_eq!(vcns, [0, 1, 2, 3, 4, 5, 6]);

    let in_use = entries(&records, false);
    assert_eq!(in_use.len(), 60);
    assert!(in_use.iter().all(|e| !e.from_slack
        && e.parent.map(|p| p.entry) == Some(64)
        && e.file_reference.is_some()));
    let mut names: Vec<&str> = in_use.iter().map(|e| e.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names[..3], ["app-003.log", "app-005.log", "app-007.log"]);
    assert!(names.contains(&"renamed-001.log") && names.contains(&"renamed-011.log"));
    assert!(!names.contains(&"app-001.log") && !names.contains(&"app-002.log"));

    let slack = entries(&records, true);
    assert_eq!(slack.len(), 157);
    assert!(slack.iter().all(|e| e.from_slack));
    assert_eq!(slack.iter().filter(|e| e.copy_of_entry_in_use).count(), 96);
    // the names no longer in use: every other log from app-020.log, and the password list
    let mut gone: Vec<&str> = slack
        .iter()
        .filter(|e| !e.copy_of_entry_in_use)
        .map(|e| e.name.as_str())
        .collect();
    gone.sort_unstable();
    gone.dedup();
    let mut expected: Vec<String> = (20..=118)
        .step_by(2)
        .map(|n| format!("app-{n:03}.log"))
        .collect();
    expected.push("passwords.txt".to_owned());
    assert_eq!(gone, expected);
    // one whose entry header was written over, one whose parent was too
    assert!(
        slack
            .iter()
            .any(|e| e.file_reference.is_none() && e.parent.is_some())
    );
    assert!(
        slack
            .iter()
            .any(|e| e.name == "passwords.txt" && e.parent.is_none())
    );
    let deleted = slack.iter().find(|e| e.name == "app-022.log").unwrap();
    assert_eq!(deleted.file_reference.map(|f| f.entry), Some(87));
    // "line 22\n"
    assert_eq!((deleted.real_size, deleted.allocated_size), (8, 8));
    assert!(deleted.created <= deleted.modified);
}

#[test]
fn japanese_names_and_a_renamed_photo() {
    let records = index_records(&sample("Photos_I30"));
    assert_eq!(records.len(), 1);
    let names: Vec<&str> = records[0].entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        ["さくら.jpg", "京都の秋.jpg", "大阪城.jpg", "東京タワー.jpg"]
    );
    let slack: Vec<(&str, bool, Option<u64>)> = records[0]
        .slack
        .iter()
        .map(|e| {
            (
                e.name.as_str(),
                e.copy_of_entry_in_use,
                e.parent.map(|p| p.entry),
            )
        })
        .collect();
    // the renamed photo's old name
    assert_eq!(
        slack,
        [("東京タワー.jpg", true, None), ("桜.jpg", false, Some(65))]
    );
}

/// A small deterministic generator (the same damage on every run).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

#[test]
fn damaged_indexes_do_not_panic() {
    let base = sample("Logs_I30");
    let mut rng = Lcg(0x1530);
    for round in 0..256 {
        let mut data = base.clone();
        for _ in 0..(1 + round % 64) {
            let at = (rng.next() as usize) % data.len();
            data[at] = rng.next() as u8;
        }
        let read = panic::catch_unwind(AssertUnwindSafe(|| index_records(&data)));
        assert!(read.is_ok(), "round {round}");
    }
    // bytes of every kind after a header that holds
    for round in 0..64 {
        let mut data = base[..4096].to_vec();
        for b in &mut data[0x40..] {
            *b = rng.next() as u8;
        }
        let read = panic::catch_unwind(AssertUnwindSafe(|| index_records(&data)));
        assert!(read.is_ok(), "random round {round}");
    }
}
