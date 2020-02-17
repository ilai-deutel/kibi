#!/usr/bin/env bash

set -euxo pipefail

# Obtain the version from Cargo.toml, prepend "v".
version=$(sed -n 's/version *= *"\(.*\)"/v\1/p' Cargo.toml)

if [ "$version" != "$TRAVIS_TAG" ]; then
  echo "Version in Cargo.toml ($version) and tag ($TRAVIS_TAG) should be equal." >&2
  exit 1
fi

cargo build --bins --target "$TARGET" --release --verbose

tempdir=$(mktemp -d 2>/dev/null || mktemp -d -t tmp)
package_name="kibi-$version-$TARGET"

mkdir "$tempdir/$package_name"

cp "target/$TARGET/release/kibi" "$tempdir/$package_name/"
strip "$tempdir/$package_name/kibi"

cp -r README.md COPYRIGHT LICENSE-APACHE LICENSE-MIT config_example "$tempdir/$package_name/"

tar czvf "$package_name.tar.gz" -C "$tempdir/$package_name" .

rm -rv "$tempdir"
