mod fixtures;

use fixtures::*;

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use tempfile::tempdir;

#[test]
fn it_respects_directory_output() {
    let d = tempdir().unwrap();
    let f = d.as_ref().join("test.out");

    let sample = mft_sample();

    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg("-f").arg(f.as_os_str()).arg(sample.as_os_str());

    assert!(
        cmd.output().unwrap().stdout.is_empty(),
        "Expected output to be printed to file, but was printed to stdout"
    );

    let mut expected = vec![];

    File::open(&f).unwrap().read_to_end(&mut expected).unwrap();
    assert!(
        !expected.is_empty(),
        "Expected output to be printed to file"
    )
}

#[test]
fn test_it_refuses_to_overwrite_directory() {
    let d = tempdir().unwrap();

    let sample = mft_sample();
    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg("-f").arg(d.path()).arg(sample.as_os_str());

    cmd.assert().failure().code(1);
}

#[test]
fn test_non_mft_file_is_error() {
    let d = tempdir().unwrap();

    let f = d.as_ref().join("test.out");

    let mut file = File::create(&f).unwrap();
    file.write_all(b"I'm a file!").unwrap();

    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg(f.as_os_str());

    cmd.assert().failure().code(1);
}

#[test]
fn test_it_exports_resident_streams() {
    let d = tempdir().unwrap();

    let sample = mft_sample();
    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg("-e").arg(d.path()).arg(sample.as_os_str());

    cmd.assert().success();

    assert_eq!(fs::read_dir(d.path()).unwrap().count(), 2142)
}

#[test]
fn resident_streams_get_the_same_names_on_every_run() {
    let names = |dir: &std::path::Path| {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        names.sort();
        names
    };
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    for d in [&a, &b] {
        let mut cmd = cargo_bin_cmd!("mft_dump");
        cmd.arg("-o")
            .arg("csv")
            .arg("-e")
            .arg(d.path())
            .arg(mft_sample().as_os_str());
        cmd.assert().success();
    }
    let first = names(a.path());
    assert_eq!(first.len(), 2142);
    assert_eq!(first, names(b.path()));
    // every file is directly inside the output directory and is named after its entry
    assert!(
        first
            .iter()
            .all(|n| n.ends_with(".dontrun") && n.contains("__"))
    );
    assert!(
        first
            .iter()
            .any(|n| n.starts_with("Program Files_AIM6_") && n.contains("__10015-1_0_"))
    );

    // a second extraction into the same directory does not overwrite evidence
    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg("-o")
        .arg("csv")
        .arg("-e")
        .arg(a.path())
        .arg(mft_sample().as_os_str());
    cmd.assert().failure().code(1);
}

#[test]
fn damaged_mfts_do_not_crash() {
    let d = tempdir().unwrap();
    let sample = fs::read(mft_sample()).unwrap();

    // the first header is all zeros (the entry size was divided by zero)
    let zero = d.as_ref().join("zero");
    fs::write(&zero, vec![0u8; 4096]).unwrap();
    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg(zero.as_os_str());
    cmd.assert().failure().code(1);

    // the first entry is wiped: the size comes from the next valid header
    let mut wiped = sample.clone();
    wiped[..1024].fill(0);
    let path = d.as_ref().join("wiped");
    fs::write(&path, &wiped).unwrap();
    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg("-o").arg("csv").arg(path.as_os_str());
    let out = cmd.assert().success().get_output().stdout.clone();
    assert!(String::from_utf8_lossy(&out).lines().count() > 13000);

    // an update sequence array that points past the entry
    let mut fixup = sample.clone();
    let e = 40 * 1024;
    fixup[e + 4..e + 6].copy_from_slice(&0x3F0u16.to_le_bytes());
    fixup[e + 6..e + 8].copy_from_slice(&40u16.to_le_bytes());
    let path = d.as_ref().join("fixup");
    fs::write(&path, &fixup).unwrap();
    let mut cmd = cargo_bin_cmd!("mft_dump");
    cmd.arg("-o").arg("csv").arg(path.as_os_str());
    cmd.assert().success();
}
