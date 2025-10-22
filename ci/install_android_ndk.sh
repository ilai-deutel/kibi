#!/usr/bin/env bash

set -euxo pipefail

DESTDIR=$1

# https://developer.android.com/ndk/downloads#lts-downloads
PACKAGE_URL='https://dl.google.com/android/repository/android-ndk-r27d-linux.zip'
SHA1SUM='22105e410cf29afcf163760cc95522b9fb981121'

TMPDIR=$(mktemp --directory)

curl "$PACKAGE_URL" \
  --output "$TMPDIR/ndk.zip" \
  --fail \
  --create-dirs
echo "$SHA1SUM" "$TMPDIR/ndk.zip" | sha1sum --check
unzip "$TMPDIR/ndk.zip" 'android-ndk-*' -d "$DESTDIR"
mv "$DESTDIR"/android-ndk-*/* "$DESTDIR"
rmdir "$DESTDIR"/android-ndk-*

rm --recursive "$TMPDIR"
