#!/usr/bin/env bash

set -euo pipefail

add_license_information() {
  font_family="$1"
  font_license_details="$2"

  # REUSE-IgnoreStart
  file_header='<!--
    SPDX-License-Identifier: MIT or Apache-2.0
    SPDX-FileCopyrightText: 2020 Ilaï Deutel
  -->'

  font_snippet_begin="<!--
    SPDX-SnippetBegin
    $font_license_details
  -->"
  # REUSE-IgnoreEnd

  font_snippet_end='<!-- SPDX-SnippetEnd -->'

  awk \
    -v header="$file_header" \
    -v font_snippet_begin="$font_snippet_begin" \
    -v font_snippet_end="$font_snippet_end" \
    'BEGIN { RS="^$" }
     !sub(/@font-face[[:space:]]*\{[[:space:]]*font-family:[[:space:]]*'"$font_family"';[^}]*\}/, font_snippet_begin "&" font_snippet_end) {
       exit 1
     }
     {
       print header "\n" $0
     }'
}