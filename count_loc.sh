#!/usr/bin/env bash

# Count the number of lines of code in kibi source code, display a result
# table. Exit code 0 will be returned if the total LOC count is below 1024.

# The lines of codes are counted using `tokei`, after removing the following
# from the code:
#   * Clippy directives
#   * Anything after #[cfg(test)]

set -euo pipefail

declare -i file_loc total_loc left_col_width
declare -A file_locs per_platform_total_locs

paths=("$(dirname "${BASH_SOURCE[0]:-$0}")/src"/*.rs)

left_col_width=6
per_platform_total_locs['unix']=0
per_platform_total_locs['windows']=0

for path in "${paths[@]}"; do
  if (( ${#path} > left_col_width )); then left_col_width=${#path}; fi;

  tempfile=$(mktemp --suffix .rs)
  # Ignore Clippy directives
  code=$(grep -v -P '^\s*#!?\[(?:allow|warn|deny)\(clippy::' "${path}")
  # Ignore everything after #[cfg(test)]
  echo "${code%'#[cfg(test)]'*}" > "${tempfile}"
  file_loc=$(tokei "${tempfile}" -t=Rust -o json | jq .Rust.code)
  rm "${tempfile}"

  file_locs[${path}]=${file_loc}

  if [[ "${path}" == "./src/unix.rs" ]]; then
    per_platform_total_locs['unix']=$((per_platform_total_locs['unix'] + file_loc))
  elif [[ "${path}" == "./src/windows.rs" ]]; then
    per_platform_total_locs['windows']=$((per_platform_total_locs['windows'] + file_loc))
  else
    for platform in "${!per_platform_total_locs[@]}"; do
      per_platform_total_locs[${platform}]=$((per_platform_total_locs[${platform}] + file_loc))
    done
  fi
done

for path in "${paths[@]}"; do
  printf "%-${left_col_width}s %4i\n" "${path}" "${file_locs[${path}]}"
done

loc_too_high=false
for platform in "${!per_platform_total_locs[@]}"; do
  total_loc=${per_platform_total_locs[${platform}]}
  printf "%b%-${left_col_width}s %4i %b" '\x1b[1m' "Total (${platform})" "${total_loc}" '\x1b[0m'
  if [[ ${total_loc} -gt 1024 ]]; then
    echo -e ' \x1b[31m(> 1024)\x1b[0m'
    loc_too_high=true
  else
    echo -e ' \x1b[32m(≤ 1024)\x1b[0m'
  fi
done

if [[ ${loc_too_high} = true ]]; then
  exit 1
fi