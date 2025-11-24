#!/usr/bin/env bash

set -euo pipefail

font_family='Fira Code'
# REUSE-IgnoreStart
font_license_details='SPDX-SnippetCopyrightText: (c) 2014, The Fira Code Project Authors (https://github.com/tonsky/FiraCode)
    SPDX-SnippetComment: https://raw.githubusercontent.com/githubnext/monaspace/refs/heads/main/LICENSE
    SPDX-License-Identifier: OFL-1.1-no-RFN'
# REUSE-IgnoreEnd

main() {
  # ls /run/
  cd "$(git rev-parse --show-toplevel)"

  source scripts/license_utils.sh

  tmp_dir=$(mktemp -d)

  # Generate animated SVG
  Rscript scripts/generate_recording.R "$tmp_dir/recording.svg"
  svgo "$tmp_dir/recording.svg" --output - \
  | add_license_information "$font_family" "$font_license_details" \
  > assets/recording.svg

  # Generate AV1 video
  agg \
    --last-frame-duration 1 \
    --cols 106 \
    --font-family 'Monaspace Neon' \
    --font-size 16 \
    --theme monokai \
    --renderer fontdue \
    --verbose \
    assets/recording.cast "$tmp_dir/recording.gif"
  ffmpeg \
    -i "$tmp_dir/recording.gif" \
    -c:v libsvtav1 \
    -crf 30 \
    -preset 0 \
    -svtav1-params keyint=10s:tune=0:enable-overlays=1:scd=1:scm=1 \
    -loop 0 \
    -cpu-used 8 \
    -an -y \
    assets/recording.webm

  rm -r "$tmp_dir"
}

main "$@"