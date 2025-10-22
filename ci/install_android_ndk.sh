#!/usr/bin/env bash

set -euxo pipefail

DESTDIR=$1

# https://developer.android.com/ndk/downloads#lts-downloads
PACKAGE_URL='https://dl.google.com/android/repository/android-ndk-r29-linux.zip'
# SHA1 Checksum of the package, from the Android Developers website
SHA1SUM='87e2bb7e9be5d6a1c6cdf5ec40dd4e0c6d07c30b'  # DevSkim: ignore DS126858,DS173237

TMPDIR=$(mktemp --directory)

curl "$PACKAGE_URL" \
  --output "$TMPDIR/ndk.zip" \
  --fail \
  --create-dirs
echo "$SHA1SUM" "$TMPDIR/ndk.zip" | sha1sum --check  # DevSkim: ignore DS126858,DS173237
unzip "$TMPDIR/ndk.zip" 'android-ndk-*' -d "$DESTDIR"
mv "$DESTDIR"/android-ndk-*/* "$DESTDIR"
rmdir "$DESTDIR"/android-ndk-*

rm --recursive "$TMPDIR"
