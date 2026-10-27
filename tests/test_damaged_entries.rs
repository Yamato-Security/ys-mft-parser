//! Damaged or crafted entries must never panic: random byte damage in real entries is parsed,
//! serialized and turned into CSV rows with full paths. `MFT_DAMAGE_ROUNDS` runs more rounds.

mod fixtures;

use fixtures::*;
use mft::MftParser;
use mft::csv::FlatMftEntryWithName;
use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

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
fn random_damage_does_not_panic() {
    let rounds: u64 = std::env::var("MFT_DAMAGE_ROUNDS")
        .ok()
        .and_then(|r| r.parse().ok())
        .unwrap_or(16);
    let sample = std::fs::read(mft_sample()).unwrap();
    let base = &sample[..2048 * 1024];

    // collect panics by location instead of stopping at the first one
    let panics: Arc<Mutex<BTreeMap<String, usize>>> = Arc::default();
    let sink = Arc::clone(&panics);
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let at = info
            .location()
            .map_or_else(|| "?".to_string(), |l| format!("{}:{}", l.file(), l.line()));
        *sink.lock().unwrap().entry(at).or_default() += 1;
    }));

    let mut rng = Lcg(0x5EED_u64);
    for _ in 0..rounds {
        let mut data = base.to_vec();
        // damage a few bytes in each of 64 random entries
        for _ in 0..64 {
            let entry = (rng.next() as usize % 2048) * 1024;
            for _ in 0..(1 + rng.next() % 8) {
                let at = entry + rng.next() as usize % 1024;
                data[at] = rng.next() as u8;
            }
        }
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let Ok(mut parser) = MftParser::from_buffer(data) else {
                return;
            };
            for i in 0..parser.get_entry_count() {
                let Ok(entry) = parser.get_entry(i) else {
                    continue;
                };
                let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                    for attribute in entry.iter_attributes() {
                        let _ = attribute.map(|a| serde_json::to_string(&a));
                    }
                    let _ = serde_json::to_string(&entry);
                    let row = FlatMftEntryWithName::from_entry(&entry, &mut parser);
                    let _ = serde_json::to_string(&row);
                }));
            }
        }));
    }
    panic::set_hook(previous);
    let panics = panics.lock().unwrap();
    assert!(panics.is_empty(), "panics by location: {panics:#?}");
}

/// Every byte of the used part of real entries set, one at a time, to 0x00 and 0xFF: the
/// extreme values of every header, attribute and data-run field.
#[test]
fn every_byte_at_its_extremes_does_not_panic() {
    let sample = std::fs::read(mft_sample()).unwrap();
    let entries: u64 = std::env::var("MFT_DAMAGE_ENTRIES")
        .ok()
        .and_then(|r| r.parse().ok())
        .unwrap_or(24);
    let base = &sample[..64 * 1024];

    let panics: Arc<Mutex<BTreeMap<String, usize>>> = Arc::default();
    let sink = Arc::clone(&panics);
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let at = info
            .location()
            .map_or_else(|| "?".to_string(), |l| format!("{}:{}", l.file(), l.line()));
        *sink.lock().unwrap().entry(at).or_default() += 1;
    }));

    for e in 0..entries.min(64) as usize {
        let start = e * 1024;
        let used = u32::from_le_bytes(base[start + 0x18..start + 0x1C].try_into().unwrap())
            .min(1024) as usize;
        for at in start..start + used {
            for value in [0x00, 0xFF] {
                if base[at] == value {
                    continue;
                }
                let mut data = base.to_vec();
                data[at] = value;
                let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                    let Ok(mut parser) = MftParser::from_buffer(data) else {
                        return;
                    };
                    if let Ok(entry) = parser.get_entry(e as u64) {
                        for attribute in entry.iter_attributes() {
                            let _ = attribute.map(|a| serde_json::to_string(&a));
                        }
                        let _ = serde_json::to_string(&entry);
                        let row = FlatMftEntryWithName::from_entry(&entry, &mut parser);
                        let _ = serde_json::to_string(&row);
                    }
                }));
            }
        }
    }
    panic::set_hook(previous);
    let panics = panics.lock().unwrap();
    assert!(panics.is_empty(), "panics by location: {panics:#?}");
}
