#!/usr/bin/env bash

set -euo pipefail

# Returns the AppStream platform triplet [1] corresponding to a Rust
# triplet [2] used in Kibi.

# [1]: https://github.com/ximion/appstream/blob/main/data/platforms.yml
# [2]: https://doc.rust-lang.org/nightly/rustc/platform-support.html
get_appstream_platform_for_rust_triplet() {
  case "$1" in
    aarch64-apple-darwin) echo aarch64-darwin-any ;;
    aarch64-pc-windows-msvc) echo aarch64-windows-msvc ;;
    aarch64-unknown-linux-gnu) echo aarch64-linux-gnu ;;
    aarch64-unknown-linux-musl) echo aarch64-linux-musl ;;
    i686-pc-windows-msvc) echo i386-windows-msvc ;;
    i686-unknown-linux-gnu) echo i386-linux-gnu ;;
    wasm32-wasip1) echo wasm32-any-any ;;
    x86_64-apple-darwin) echo x86_64-darwin-any ;;
    x86_64-pc-windows-gnu) echo x86_64-windows-gnu ;;
    x86_64-pc-windows-msvc) echo x86_64-windows-msvc ;;
    x86_64-unknown-linux-gnu) echo x86_64-linux-gnu ;;
    x86_64-unknown-linux-musl) echo x86_64-linux-musl ;;
    *) echo "$1" ;;
  esac
}

generate_appstream_artifact_xml() {
  type=$1
  asset=$2
  url=$3

  if [[ "$type" == 'binary' ]]; then
    rust_target=$(sed -E "s/kibi-[^-]+-([^\.]+)\..+/\1/" <<< "$(basename "$asset")")
    platform=$(get_appstream_platform_for_rust_triplet "$rust_target")
    echo "<artifact type=\"binary\" platform=\"$platform\">"
  else
    echo "<artifact type=\"$type\">"
  fi
  
  echo "<location>$url</location>
    <checksum type=\"blake3\">$(b3sum "$asset" | cut -d " " -f 1)</checksum>
    <checksum type=\"blake2b\">$(b2sum "$asset" | cut -d " " -f 1)</checksum>
    <checksum type=\"sha512\">$(sha512sum "$asset" | cut -d " " -f 1)</checksum>
    <checksum type=\"sha256\">$(sha256sum "$asset" | cut -d " " -f 1)</checksum>
    <checksum type=\"sha1\">$(sha1sum "$asset" | cut -d " " -f 1)</checksum>
    <size type=\"download\">$(du -b "$asset" | cut -f 1)</size>"
  
  if [[ "$asset" == *.tar.gz ]]; then
    size_installed=$(gzip -l "$asset" | tail -n 1 | xargs | cut -d " " -f 2)
    echo "<size type=\"installed\">$size_installed</size>"
  elif [[ "$asset" == *.zip ]]; then
    size_installed=$(unzip -l "$asset" | tail -n 1 | xargs | cut -d " " -f 1)
    echo "<size type=\"installed\">$size_installed</size>"
  fi
  
  echo "</artifact>"
}

main() {
  if [ -z "${1+x}" ]; then
    echo >&2 "Usage: $(basename "$0") <tag>"
    exit 1
  fi

  tag="$1"

  if [[ $(gh release view "$tag" --json isDraft | jq -c .isDraft) == "false" ]]; then
    echo >&2 "Release $tag is not a draft"
    exit 1
  fi

  tmpdir=$(mktemp -d)

  mkdir "$tmpdir/assets" "$tmpdir/outputs"
  # shellcheck disable=SC2064
  trap "rm -r $tmpdir/assets" EXIT
  gh release download "$tag" --dir "$tmpdir/assets"

  # Verification
  for asset in "$tmpdir"/assets/*; do
    if [[ ! -f "$asset" ]]; then
      echo >&2 "No assets found"
      exit 1
    fi
    if [[ "$asset" == *.intoto.jsonl || "$asset" == *.asc ]]; then
      continue
    fi
    if [ ! -f "$asset.intoto.jsonl" ]; then
      echo >&2 "No attestation found for $asset"
      exit 1
    fi

    slsa-verifier verify-artifact "$asset" \
      --provenance-path "$asset.intoto.jsonl"\
      --source-uri github.com/ilai-deutel/kibi \
      --source-versioned-tag "$tag"
  done

  # Signing
  for asset in "$tmpdir"/assets/*; do
    if [[ "$asset" == *.intoto.jsonl || "$asset" == *.asc ]]; then
      continue
    fi
    gpg --local-user 102588418FF7E165696490A206E8A973494808A2 --armor --detach-sign --verbose "$asset"
    mv "$asset.asc" "$tmpdir/outputs"
  done

  # Summary
  echo '## Packaged binaries

  | Asset | Platform | Checksums | Signature | Attestation (SLSA) | Transparency log |
  | - | - | -  | - | - | - |' > "$tmpdir/summary.md"
  for asset in "$tmpdir"/assets/*; do
    if [[ "$asset" == *.intoto.jsonl || "$asset" == *.asc ]]; then
      continue
    fi
    name=$(basename "$asset")
    target=$(sed -E "s/kibi-$tag-([^\.]+)\..+/\1/" <<< "$name")
    formatted_asset="[\`$name\`](https://github.com/ilai-deutel/kibi/releases/download/$tag/$name)"
    platform=$(rustc +nightly -Z unstable-options --print target-spec-json --target "$target" | jq -r '.metadata.description')
    checksums="<details><summary>Checksums</summary>BLAKE3: \`$(b3sum "$asset" | cut -d " " -f 1)\`<br />BLAKE2b: \`$(b2sum "$asset" | cut -d " " -f 1)\`<br />SHA-512: \`$(sha512sum "$asset" | cut -d " " -f 1)\`<br />SHA-256: \`$(sha256sum "$asset" | cut -d " " -f 1)\`<br />SHA-1: \`$(sha1sum "$asset" | cut -d " " -f 1)\`</details>"
    signature="[\`$name.asc\`](https://github.com/ilai-deutel/kibi/releases/download/$tag/$name.asc)"
    formatted_slsa="[$name.intoto.jsonl](https://github.com/ilai-deutel/kibi/releases/download/$tag/$name.intoto.jsonl)"
    log_index=$(jq -r ".verificationMaterial.tlogEntries[].logIndex" "$asset.intoto.jsonl")
    transparency_log="[rekor:$log_index](https://search.sigstore.dev/?logIndex=$log_index)"
    echo "| $formatted_asset | $platform | $checksums | $signature | $formatted_slsa | $transparency_log"  >> "$tmpdir/summary.md"
  done
  mdformat "$tmpdir/summary.md"

  # Metainfo summary
  echo '<artifacts>' > "$tmpdir/metainfo-artifacts.xml"
  curl --location "https://github.com/ilai-deutel/kibi/archive/refs/tags/$tag.tar.gz" --output "$tmpdir/source.tar.gz"
  generate_appstream_artifact_xml source "$tmpdir/source.tar.gz" "https://github.com/ilai-deutel/kibi/archive/refs/tags/$tag.tar.gz"  >> "$tmpdir/metainfo-artifacts.xml"
  for asset in "$tmpdir"/assets/*; do
    if [[ "$asset" == *.tar.gz || "$asset" == *.zip ]]; then
      generate_appstream_artifact_xml binary "$asset" "https://github.com/ilai-deutel/kibi/releases/download/$tag/$(basename "$asset")" >> "$tmpdir/metainfo-artifacts.xml"
    fi
  done
  echo '</artifacts>' >> "$tmpdir/metainfo-artifacts.xml"
  xmllint --format "$tmpdir/metainfo-artifacts.xml" --output "$tmpdir/metainfo-artifacts.xml"

  echo >&2 "New assets: $tmpdir/outputs"
  echo >&2 "Summary: $tmpdir/summary.md"
  echo >&2 "Metainfo artifacts: $tmpdir/metainfo-artifacts.xml"
}

main "$@"