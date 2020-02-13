#!/usr/bin/env bash

set -euxo pipefail

cargo build --bins --target "$TARGET" --release --verbose

tempdir=$(mktemp -d 2>/dev/null || mktemp -d -t tmp)
package_name="kibi-$TRAVIS_TAG-$TARGET"

mkdir "$tempdir/$package_name"

cp "target/$TARGET/release/kibi" "$tempdir/$package_name/"
strip "$tempdir/$package_name/kibi"

cp -r README.md COPYRIGHT LICENSE-APACHE LICENSE-MIT config_example "$tempdir/$package_name/"

tar czvf "$package_name.tar.gz" -C "$tempdir/$package_name" .

rm -rv "$tempdir"
