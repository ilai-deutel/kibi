#!/usr/bin/env bash

set -euo pipefail

font_family='Monaspace Neon'
# REUSE-IgnoreStart
font_license_details='SPDX-SnippetCopyrightText: (c) 2023, GitHub https://github.com/githubnext/monaspace
    SPDX-SnippetComment: https://raw.githubusercontent.com/githubnext/monaspace/refs/heads/main/LICENSE
    SPDX-License-Identifier: OFL-1.1-RFN'
# REUSE-IgnoreEnd

generate_screenshot() {
  mode="$1"
  kibi_version=$(cargo pkgid | cut -d '#' -f2)
  XDG_CONFIG_HOME="$(pwd)/scripts" termframe \
    --title "Kibi v$kibi_version" \
    --font-size 14 \
    --mode "$mode" \
    -- ./target/debug/kibi src/editor.rs
}

main() {
  cd "$(git rev-parse --show-toplevel)"

  source scripts/license_utils.sh

  cargo build

  for mode in 'dark' 'light'; do
    generate_screenshot "$mode" | svgo - | add_license_information "$font_family" "$font_license_details" > "assets/screenshot-$mode.svg"
  done
}

main "$@"