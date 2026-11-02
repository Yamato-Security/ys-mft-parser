// Copyright (C) 2026 Shirofune Security Co., Ltd.
// Copyright (c) 2019 Omer Ben-Amram
//
// This file is changed from omerbenamram/mft (https://github.com/omerbenamram/mft), whose code is
// under the MIT License or the Apache License 2.0 (LICENSE-MIT, LICENSE-APACHE).
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use mft::csv::FlatMftEntryWithName;
use mft::{MftEntry, MftParser};
use std::hint::black_box;
use std::io::Cursor;

const SAMPLE: &[u8] = include_bytes!("../../samples/MFT");

/// A parser over the in-memory sample, without copying it.
fn parser() -> MftParser<Cursor<&'static [u8]>> {
    MftParser::from_read_seek(Cursor::new(SAMPLE), Some(SAMPLE.len() as u64)).unwrap()
}

fn all_entries() -> Vec<MftEntry> {
    parser().iter_entries().filter_map(Result::ok).collect()
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    group.throughput(Throughput::Bytes(SAMPLE.len() as u64));

    group.bench_function("entries", |b| {
        b.iter(|| {
            let mut parser = parser();
            for entry in parser.iter_entries() {
                let _ = black_box(entry);
            }
        })
    });

    group.bench_function("entries_and_attributes", |b| {
        b.iter(|| {
            let mut parser = parser();
            let mut count = 0usize;
            for entry in parser.iter_entries().filter_map(Result::ok) {
                for attr in entry.iter_attributes() {
                    count += black_box(attr).is_ok() as usize;
                }
            }
            count
        })
    });

    group.finish();
}

fn bench_serialize(c: &mut Criterion) {
    let entries = all_entries();

    let mut group = c.benchmark_group("serialize");
    group.throughput(Throughput::Elements(entries.len() as u64));

    group.bench_function("json", |b| {
        let mut buf = Vec::with_capacity(64 * 1024 * 1024);
        b.iter(|| {
            buf.clear();
            for entry in &entries {
                serde_json::to_writer(&mut buf, entry).unwrap();
                buf.push(b'\n');
            }
            buf.len()
        })
    });

    group.bench_function("csv", |b| {
        let mut parser = parser();
        b.iter(|| {
            let mut writer = csv::Writer::from_writer(Vec::with_capacity(8 * 1024 * 1024));
            for entry in &entries {
                writer
                    .serialize(FlatMftEntryWithName::from_entry(entry, &mut parser))
                    .unwrap();
            }
            writer.into_inner().unwrap().len()
        })
    });

    group.finish();
}

fn bench_full_path(c: &mut Criterion) {
    let entries = all_entries();

    let mut group = c.benchmark_group("full_path");
    group.throughput(Throughput::Elements(entries.len() as u64));

    // Fresh parser per iteration: measures resolution including cache misses.
    group.bench_function("cold_cache", |b| {
        b.iter(|| {
            let mut parser = parser();
            for entry in &entries {
                black_box(parser.get_full_path_for_entry(entry).unwrap());
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_parse, bench_serialize, bench_full_path);
criterion_main!(benches);
