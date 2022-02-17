#!/usr/bin/env bash

# Count the number of lines of code in kibi source code, display a result
# table. Exit code 0 will be returned if the total LOC count is below 1024.

# The lines of codes are counted using `tokei`, after removing the following
# from the code:
#   * Clippy directives
#   * Anything after #[cfg(test)]

set -euo pipefail

declare -i file_loc total_loc left_col_width
declare -A file_locs

paths=("$(dirname "${BASH_SOURCE[0]:-$0}")/src"/*.rs)

left_col_width=6
total_loc=0

for path in "${paths[@]}"; do
  if (( ${#path} > left_col_width )); then left_col_width=${#path}; fi;

  tempfile=$(mktemp --suffix .rs)
  # Ignore Clippy directives
  code=$(grep -v -P '^\s*#!?\[(?:allow|warn|deny)\(clippy::' "${path}")
  # Ignore everything after #[cfg(test)]
  echo "${code%'#[cfg(test)]'*}" > "${tempfile}"
  file_loc=$(tokei "${tempfile}" -t=Rust -o json | jq .Rust.code)
  rm "${tempfile}"
  # Ignore unix, wasi and windows platform files
  # (these are only indirectly related to the program itself and thus are
  #  treated the same as library dependencies and ignored)
  if [ "${path}" == "./src/unix.rs" ]; then file_loc=0; fi
  if [ "${path}" == "./src/wasi.rs" ]; then file_loc=0; fi
  if [ "${path}" == "./src/windows.rs" ]; then file_loc=0; fi

  file_locs[${path}]=${file_loc}
  total_loc+=${file_loc}
done

for path in "${paths[@]}"; do
  printf "%-${left_col_width}s %${#total_loc}s\n" "${path}" "${file_locs[${path}]}"
done

printf "%b%-${left_col_width}s %i %b" '\x1b[1m' 'Total' "${total_loc}" '\x1b[0m'

if [[ ${total_loc} -gt 1024 ]]; then
  echo -e ' \x1b[31m(> 1024)\x1b[0m'
  exit 1
else
  echo -e ' \x1b[32m(≤ 1024)\x1b[0m'
fi
