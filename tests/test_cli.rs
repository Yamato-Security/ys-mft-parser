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

    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
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
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg("-f").arg(d.path()).arg(sample.as_os_str());

    cmd.assert().failure().code(1);
}

#[test]
fn test_non_mft_file_is_error() {
    let d = tempdir().unwrap();

    let f = d.as_ref().join("test.out");

    let mut file = File::create(&f).unwrap();
    file.write_all(b"I'm a file!").unwrap();

    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg(f.as_os_str());

    cmd.assert().failure().code(1);
}

#[test]
fn test_it_exports_resident_streams() {
    let d = tempdir().unwrap();

    let sample = mft_sample();
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg("-e").arg(d.path()).arg(sample.as_os_str());

    cmd.assert().success();

    // 2142 resident streams, 92 of which are empty and are not exported.
    assert_eq!(fs::read_dir(d.path()).unwrap().count(), 2050);
    for f in fs::read_dir(d.path()).unwrap() {
        assert!(f.unwrap().metadata().unwrap().len() > 0);
    }
}

#[test]
fn test_bodyfile_output() {
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg("-o")
        .arg("bodyfile")
        .arg("-r")
        .arg("0,8671,11600")
        .arg(mft_sample());
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let thumbs =
        "/Documents and Settings/All Users/Documents/My Pictures/Sample Pictures/Thumbs.db";
    let gif = "/Documents and Settings/Donald Blake/Local Settings/Temporary Internet Files/Content.IE5/B70NJDCZ/online[1].gif";
    // Same layout as `fls -m /`: a `$FILE_NAME` line, then one line per `$DATA` stream.
    let expected = [
        "0|/$MFT ($FILE_NAME)|0-48-3|r/rr-xr-xr-x|0|0|74|1183207852|1183207852|1183207852|1183207852".to_string(),
        "0|/$MFT|0-128-1|r/rr-xr-xr-x|0|0|13402112|1183207852|1183207852|1183207852|1183207852".to_string(),
        format!("0|{thumbs} ($FILE_NAME)|8671-48-2|r/rr-xr-xr-x|0|0|84|1183244062|1183244062|1183244062|1183244062"),
        format!("0|{thumbs}|8671-128-3|r/rr-xr-xr-x|0|0|7680|1183244062|1183244062|1183244062|1183244062"),
        format!("0|{thumbs}:encryptable|8671-128-4|r/rr-xr-xr-x|0|0|0|1183244062|1183244062|1183244062|1183244062"),
        format!("0|{gif} ($FILE_NAME) (deleted)|11600-48-2|-/rrwxrwxrwx|0|0|92|1232382084|1232382084|1232382084|1232382084"),
        format!("0|{gif} (deleted)|11600-128-1|-/rrwxrwxrwx|0|0|84|1232382084|1232382084|1232382084|1232382084"),
    ];

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().collect::<Vec<_>>(), expected);
}

#[test]
fn test_bodyfile_lines_have_eleven_fields() {
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg("-o").arg("bodyfile").arg(mft_sample());
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().count() > 10_000);
    for line in stdout.lines() {
        assert_eq!(
            line.split('|').count(),
            11,
            "malformed bodyfile line: {line}"
        );
    }
}

#[test]
fn test_json_output_includes_full_path() {
    for format in ["json", "jsonl"] {
        let mut cmd = cargo_bin_cmd!("ys-mft-parser");
        cmd.arg("-o")
            .arg(format)
            .arg("-r")
            .arg("0,8671")
            .arg(mft_sample());
        let output = cmd.output().unwrap();
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).unwrap();
        let paths: Vec<String> = serde_json::Deserializer::from_str(&stdout)
            .into_iter::<serde_json::Value>()
            .map(|entry| entry.unwrap()["full_path"].as_str().unwrap().to_owned())
            .collect();

        assert_eq!(
            paths,
            [
                "$MFT",
                r"Documents and Settings\All Users\Documents\My Pictures\Sample Pictures\Thumbs.db"
            ],
            "unexpected paths for -o {format}"
        );
    }
}

#[test]
fn test_bodyfile_follows_extension_records() {
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg("-o")
        .arg("bodyfile")
        .arg("-r")
        .arg("47,48")
        .arg(mft_sample_name("ntfs1-gen2.MFT"));
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // `$DATA` of both files lives in extension records (48's `$ATTRIBUTE_LIST` is non-resident).
    // Sizes match `fls -m` on `testdata/ntfs/ntfs1-gen2.E01`. `fls` renumbers the attribute ids
    // of merged attributes; these are the ids on disk.
    let stdout = String::from_utf8(output.stdout).unwrap();
    let data_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("-128-"))
        .map(|l| l.rsplitn(5, '|').last().unwrap())
        .collect();
    assert_eq!(
        data_lines,
        [
            "0|/RAW/logfile1.txt|47-128-0|r/rrwxrwxrwx|0|0|21888890",
            "0|/Compressed/logfile1.txt|48-128-0|r/rrwxrwxrwx|0|0|21888890",
        ]
    );
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
        let mut cmd = cargo_bin_cmd!("ys-mft-parser");
        cmd.arg("-o")
            .arg("csv")
            .arg("-e")
            .arg(d.path())
            .arg(mft_sample().as_os_str());
        cmd.assert().success();
    }
    let first = names(a.path());
    // 2,142 resident streams, of which 92 are empty and not written
    assert_eq!(first.len(), 2050);
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
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
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
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg(zero.as_os_str());
    cmd.assert().failure().code(1);

    // the first entry is wiped: the size comes from the next valid header
    let mut wiped = sample.clone();
    wiped[..1024].fill(0);
    let path = d.as_ref().join("wiped");
    fs::write(&path, &wiped).unwrap();
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
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
    let mut cmd = cargo_bin_cmd!("ys-mft-parser");
    cmd.arg("-o").arg("csv").arg(path.as_os_str());
    cmd.assert().success();
}
