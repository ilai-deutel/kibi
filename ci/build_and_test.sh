#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2020 Ilaï Deutel & Kibi Contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0


set -euxo pipefail

run_tests=true
enable_coverage=true

while [[ $# -gt 0 ]]; do
  case $1 in
    --target)
      target="$2"
      shift
      shift
      ;;
    --output-dir)
      output_dir="$2"
      shift
      shift
      ;;
    --no-run-tests)
      run_tests="false"
      shift
      ;;
    --no-coverage)
      enable_coverage="false"
      shift
      ;;
    --sanitizer)
      sanitizer="$2"
      shift
      shift
      ;;
    *)
      echo "Unknown argument $1"
      exit 1
      ;;
  esac
done

target=${target:-$(rustc -vV | sed -n 's/host: //p')}

export RUST_LOG=info
export RUSTFLAGS="${RUSTFLAGS:-} -D warnings"
export RUSTDOCFLAGS="${RUSTDOCFLAGS:-}"
export ZFLAGS=''

if [[ "$enable_coverage" == "true" ]]; then
  output_dir=${output_dir:-$PWD/target/coverage/$(date +%Y-%m-%d_%H-%M-%S)}
  export CARGO_INCREMENTAL=0
  export RUSTFLAGS="$RUSTFLAGS -C instrument-coverage"
  export LLVM_PROFILE_FILE="$output_dir/kibi-%p-%m.profraw"
fi

if [[ -n ${sanitizer-} ]]; then
  export RUSTFLAGS="$RUSTFLAGS -Zsanitizer=$sanitizer"
  export RUSTDOCFLAGS="$RUSTDOCFLAGS -Zsanitizer=$sanitizer"
  export ZFLAGS='-Z build-std'

  if [[ "$sanitizer" == "memory" ]]; then
    export RUSTFLAGS="$RUSTFLAGS -Zsanitizer-memory-track-origins"
    export RUSTDOCFLAGS="$RUSTDOCFLAGS -Zsanitizer-memory-track-origins"
  fi
fi

# shellcheck disable=SC2086
cargo build \
  $ZFLAGS \
  --target "$target" \
  --all-features \
  --locked \
  --verbose

if [[ "$run_tests" == "false" ]]; then
  exit 0
fi

# shellcheck disable=SC2086
cargo nextest run \
  $ZFLAGS \
  --profile ci \
  --target "$target" \
  --all-features \
  --locked \
  --show-progress counter

if [[ "$enable_coverage" == "true" ]]; then
  grcov \
    --binary-path "./target/$target/debug" \
    --source-dir . \
    --keep-only 'src/*' \
    --branch \
    --output-path "$output_dir/lcov.info" \
    --output-types lcov \
    --log-level INFO \
    "$output_dir"
fi