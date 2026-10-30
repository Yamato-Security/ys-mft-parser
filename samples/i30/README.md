# `$I30` samples

The directory indexes (`$INDEX_ALLOCATION:$I30`) of two directories of an NTFS volume that
`make_volume.sh` builds with `mkntfs` and ntfs-3g (FUSE, no root needed): files made, then
deleted or renamed, so that the indexes have entries in their slack. Made for this repository
(same licence as the code). The streams were read out of the volume with The Sleuth Kit:

```
./make_volume.sh vol.img
icat vol.img 64-160-5 > Logs_I30      # \Logs
icat vol.img 65-160-5 > Photos_I30    # \写真
```

| File | Directory | INDX records | Entries in use | In slack |
| --- | --- | --- | --- | --- |
| `Logs_I30` | `\Logs` (MFT entry 64): 120 logs made, every other one deleted, two renamed, two other files deleted | 7 | 60 | 157 (51 names no longer in use; 96 copies of entries in use) |
| `Photos_I30` | `\写真` (MFT entry 65): five photos with Japanese names, one renamed, one deleted | 1 | 4 | 2 (the renamed photo's old name; a copy of an entry in use) |

The entries in use and in slack are those dfir_ntfs reads from the same streams.
