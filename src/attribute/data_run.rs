use serde::Serialize;

// adapted from https://github.com/rkapl/ntfs-reclaim/blob/a68e87b21c12631311fc3f279f5b03bd8f23d57b/src/data_runs.rs
// original didn't support sparse clusters

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum RunType {
    Standard,
    Sparse,
}

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct DataRun {
    pub lcn_offset: u64,
    pub lcn_length: u64,
    pub run_type: RunType,
}

fn decode_run_value<T: Iterator<Item = u8>>(it: &mut T, bytes: u8) -> Option<u64> {
    if bytes == 0 || bytes > 8 {
        // nothing to shift into place (a shift by 64 bits overflows)
        return (bytes == 0).then_some(0);
    }
    let mut acc = 0u64;
    for _ in 0..bytes {
        let v = it.next()?;
        acc = (acc >> 8) | ((v as u64) << 56);
    }
    acc >>= (8 - bytes) * 8;
    Some(acc)
}

fn decode_run_svalue<T: Iterator<Item = u8>>(it: &mut T, bytes: u8) -> Option<i64> {
    if bytes == 0 {
        return Some(0);
    }
    let mut acc = decode_run_value(it, bytes)? as i64;
    // sign extend
    acc <<= (8 - bytes) * 8;
    acc >>= (8 - bytes) * 8;
    Some(acc)
}

/// Decode a runlist. Every run's LCN is stored as a signed offset from the LCN of the previous
/// run that has one (the first from 0); sparse runs have no LCN and do not move it. `None` for a
/// malformed runlist: a run without a length, more than 8 bytes for a value, or an LCN that would
/// be negative or overflow.
pub fn decode_data_runs(runs: &[u8]) -> Option<Vec<DataRun>> {
    let mut it = runs.iter().copied();
    let mut out: Vec<DataRun> = Vec::new();
    let mut lcn: u64 = 0;

    loop {
        let h = it.next()?;
        if h == 0 {
            break;
        }
        let offset_size = (h & 0xF0) >> 4;
        let length_size = h & 0x0F;
        if offset_size > 8 || length_size == 0 || length_size > 8 {
            return None;
        }
        let length = decode_run_value(&mut it, length_size)?;
        let (abs_offset, run_type) = if offset_size != 0 {
            let rel_offset = decode_run_svalue(&mut it, offset_size)?;
            lcn = lcn.checked_add_signed(rel_offset)?;
            (lcn, RunType::Standard)
        } else {
            // offset_size of 0 == sparse cluster
            (0, RunType::Sparse)
        };
        out.push(DataRun {
            lcn_offset: abs_offset,
            lcn_length: length,
            run_type,
        });
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_decode() {
        assert_eq!(
            decode_run_value(&mut vec![0x34, 0x56].into_iter(), 2),
            Some(0x5634)
        );
        assert_eq!(
            decode_run_svalue(&mut vec![0xE0].into_iter(), 1),
            Some(-0x20)
        );
        assert_eq!(decode_run_svalue(&mut vec![0xE0].into_iter(), 2), None);
        assert_eq!(decode_run_value(&mut vec![].into_iter(), 0), Some(0));
        assert_eq!(decode_run_svalue(&mut vec![].into_iter(), 0), Some(0));
    }

    fn run(lcn_offset: u64, lcn_length: u64, run_type: RunType) -> DataRun {
        DataRun {
            lcn_offset,
            lcn_length,
            run_type,
        }
    }

    #[test]
    fn runs_after_a_sparse_run_are_relative_to_the_last_lcn() {
        // 2 clusters at LCN 100, 3 sparse clusters, 1 cluster 5 after LCN 100, 4 clusters 2 before
        let runs = [
            0x11, 0x02, 0x64, 0x01, 0x03, 0x11, 0x01, 0x05, 0x11, 0x04, 0xFE, 0x00,
        ];
        assert_eq!(
            decode_data_runs(&runs),
            Some(vec![
                run(100, 2, RunType::Standard),
                run(0, 3, RunType::Sparse),
                run(105, 1, RunType::Standard),
                run(103, 4, RunType::Standard),
            ])
        );
        // a leading sparse run
        assert_eq!(
            decode_data_runs(&[0x01, 0x08, 0x11, 0x02, 0x10, 0x00]),
            Some(vec![
                run(0, 8, RunType::Sparse),
                run(16, 2, RunType::Standard)
            ])
        );
    }

    #[test]
    fn malformed_runlists_are_rejected_without_panicking() {
        // a run header without a length
        assert_eq!(decode_data_runs(&[0x10, 0x05, 0x00]), None);
        // an LCN before the start of the volume
        assert_eq!(
            decode_data_runs(&[0x11, 0x01, 0x05, 0x11, 0x01, 0xF0, 0x00]),
            None
        );
        // an LCN past u64::MAX
        let mut runs = vec![0x81, 0x01];
        runs.extend_from_slice(&i64::MAX.to_le_bytes());
        for _ in 0..3 {
            runs.extend_from_slice(&[0x81, 0x01]);
            runs.extend_from_slice(&i64::MAX.to_le_bytes());
        }
        runs.push(0x00);
        assert_eq!(decode_data_runs(&runs), None);
        // truncated
        assert_eq!(decode_data_runs(&[0x11, 0x01]), None);
    }
}
