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
  Rscript scripts/generate_recording.R "$tmp_dir/recording.svg"
  
  svgo "$tmp_dir/recording.svg" --output - \
  | add_license_information "$font_family" "$font_license_details" \
  > assets/recording.svg

  rm -r "$tmp_dir"
}

main "$@"