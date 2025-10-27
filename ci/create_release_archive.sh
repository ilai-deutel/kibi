#!/usr/bin/env bash

set -euxo pipefail

while [[ $# -gt 0 ]]; do
  case $1 in
    --target)
      target="$2"
      shift
      shift
      ;;
    --tag)
      tag="$2"
      shift
      shift
      ;;
    --dest_path)
      dest_path="$2"
      shift
      shift
      ;;
    --compress)
      compress="$2"
      shift
      shift
      ;;
    *)
      echo "Unknown argument"
      exit 1
      ;;
  esac
done

RUSTFLAGS='-D warnings' cargo build --target "$target" --release --verbose

binary_path="target/$target/release/kibi"

if [[ "$target" == 'wasm32-wasip1' ]]; then
  binary_path="${binary_path}.wasm"
elif [[ "$target" == *-windows-* ]]; then
  binary_path="${binary_path}.exe"
fi

if [[ "$compress" == "true" ]]; then
  upx --best --lzma "$binary_path"
fi

tmp_dir="$(mktemp --directory)"
# shellcheck disable=SC2064
trap "rm -r $tmp_dir" EXIT

archive_dir="kibi-$tag-$target"
mkdir "$tmp_dir/$archive_dir"
cp -R \
  "$binary_path" \
  CHANGELOG.md \
  COPYRIGHT \
  LICENSE-APACHE \
  LICENSE-MIT \
  README.md \
  config_example.ini \
  syntax.d \
  "$tmp_dir/$archive_dir"

if [[ "$dest_path" == *.tar.gz ]]; then
  tar czvf "$dest_path" -C "$tmp_dir" "$archive_dir"
elif  [[ "$dest_path" == *.zip ]]; then
  (cd "$tmp_dir"; 7z a "$dest_path" "$archive_dir")
else
  echo >&2 "Invalid extension for $dest_path"
  exit 1
fi
