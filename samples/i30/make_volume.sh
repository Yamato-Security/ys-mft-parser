#!/bin/sh
# Builds a small NTFS volume whose directory indexes ($I30) have entries in their slack: files made,
# then deleted or renamed, with ntfs-3g (FUSE, no root needed).
set -eu
img=$1
mnt=$(mktemp -d)
rm -f "$img"
truncate -s 32M "$img"
mkntfs -F -f -q -L I30TEST "$img"
ntfs-3g "$img" "$mnt"
mkdir "$mnt/Logs" "$mnt/写真"
i=1
while [ $i -le 120 ]; do
    printf 'line %d\n' "$i" > "$mnt/Logs/app-$(printf %03d $i).log"
    i=$((i + 1))
done
for name in 東京タワー 富士山 大阪城 京都の秋 桜; do
    printf '%s\n' "$name" > "$mnt/写真/$name.jpg"
done
echo secret > "$mnt/Logs/passwords.txt"
echo tool > "$mnt/Logs/mimikatz.exe"
# deleted: every other log, the password list and the tool; renamed: a few logs, a photo
i=2
while [ $i -le 120 ]; do
    rm "$mnt/Logs/app-$(printf %03d $i).log"
    i=$((i + 2))
done
rm "$mnt/Logs/passwords.txt" "$mnt/Logs/mimikatz.exe"
mv "$mnt/Logs/app-001.log" "$mnt/Logs/renamed-001.log"
mv "$mnt/Logs/app-011.log" "$mnt/Logs/renamed-011.log"
mv "$mnt/写真/桜.jpg" "$mnt/写真/さくら.jpg"
rm "$mnt/写真/富士山.jpg"
fusermount3 -u "$mnt"
rmdir "$mnt"
