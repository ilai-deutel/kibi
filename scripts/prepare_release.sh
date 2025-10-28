#!/usr/bin/env bash

set -euo pipefail

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
  if [[ "$asset" == *.intoto.jsonl ]]; then
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
  if [[ "$asset" == *.intoto.jsonl ]]; then
    continue
  fi
  gpg --local-user 102588418FF7E165696490A206E8A973494808A2 --armor --detach-sign --verbose "$asset"
  mv "$asset.asc" "$tmpdir/outputs"
done

# Summary
echo '## Packaged binaries

| Asset | Platform | Checksum | Signature | Attestation (SLSA) | Transparency log |
| - | - | -  | - | - | - |' > "$tmpdir/summary.md"
for asset in "$tmpdir"/assets/*; do
  if [[ "$asset" == *.intoto.jsonl ]]; then
    continue
  fi
  name=$(basename "$asset")
  target=$(sed -E "s/kibi-$tag-([^\.]+)\..+/\1/" <<< "$name")
  formatted_asset="[\`$name\`](https://github.com/ilai-deutel/kibi/releases/download/$tag/$name)"
  platform=$(rustc +nightly -Z unstable-options --print target-spec-json --target "$target" | jq -r '.metadata.description')
  checksum="sha256:$(sha256sum "$asset" | cut -d " " -f 1)"
  signature="[\`$name.asc\`](https://github.com/ilai-deutel/kibi/releases/download/$tag/$name.asc)"
  formatted_slsa="[$name.intoto.jsonl](https://github.com/ilai-deutel/kibi/releases/download/$tag/$name.intoto.jsonl)"
  log_index=$(jq -r ".verificationMaterial.tlogEntries[].logIndex" "$asset.intoto.jsonl")
  transparency_log="[rekor:$log_index](https://search.sigstore.dev/?logIndex=$log_index)"
  echo "| $formatted_asset | $platform | $checksum | $signature | $formatted_slsa | $transparency_log"  >> "$tmpdir/summary.md"
done
mdformat "$tmpdir/summary.md"

echo >&2 "New assets: $tmpdir/outputs"
echo >&2 "Summary: $tmpdir/summary.md"
