#!/usr/bin/env python3
"""Replaces the Git LFS pointer files under testdata/ with their content from upstream.

This fork doesn't store upstream's LFS objects (they would count against Yamato Security's LFS
quota on every CI run), so the disk images that the ntfs, ewf and aff crates test with are
downloaded from omerbenamram/mft's LFS store by object ID, and each one is checked against the
size and SHA-256 its pointer file names. Run it from the repository root; it only uses the
standard library.
"""

import hashlib
import json
import pathlib
import sys
import urllib.request

LFS_BATCH = "https://github.com/omerbenamram/mft.git/info/lfs/objects/batch"
POINTER_START = b"version https://git-lfs.github.com/spec/v1\n"


def pointers(root):
    """(path, oid, size) of every LFS pointer file under root."""
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.stat().st_size > 1024:
            continue
        data = path.read_bytes()
        if not data.startswith(POINTER_START):
            continue
        fields = dict(line.split(" ", 1) for line in data.decode().splitlines()[1:] if " " in line)
        yield path, fields["oid"].removeprefix("sha256:"), int(fields["size"])


def request(url, body=None, headers=None):
    req = urllib.request.Request(url, data=body, headers=headers or {})
    return urllib.request.urlopen(req, timeout=300)


def main():
    files = list(pointers(pathlib.Path("testdata")))
    if not files:
        print("no LFS pointer files under testdata/")
        return 0
    body = json.dumps({
        "operation": "download",
        "transfers": ["basic"],
        "objects": [{"oid": oid, "size": size} for _, oid, size in files],
    }).encode()
    lfs_json = "application/vnd.git-lfs+json"
    with request(LFS_BATCH, body, {"Accept": lfs_json, "Content-Type": lfs_json}) as resp:
        objects = {o["oid"]: o for o in json.load(resp)["objects"]}
    failed = False
    for path, oid, size in files:
        obj = objects.get(oid, {})
        download = obj.get("actions", {}).get("download")
        if download is None:
            print(f"{path}: not available upstream ({obj.get('error')})", file=sys.stderr)
            failed = True
            continue
        digest = hashlib.sha256()
        tmp = path.with_name(path.name + ".part")
        with request(download["href"], headers=download.get("header", {})) as resp, open(tmp, "wb") as out:
            while chunk := resp.read(1 << 20):
                digest.update(chunk)
                out.write(chunk)
        if tmp.stat().st_size != size or digest.hexdigest() != oid:
            tmp.unlink()
            print(f"{path}: size or SHA-256 doesn't match its pointer", file=sys.stderr)
            failed = True
            continue
        tmp.replace(path)
        print(f"{path}: {size:,} bytes")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
