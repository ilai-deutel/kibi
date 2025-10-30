#!/usr/bin/env bash

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
    *)
      echo "Unknown argument $1"
      exit 1
      ;;
  esac
done

target=${target:-$(rustc -vV | sed -n 's/host: //p')}

export RUST_LOG=info
export RUSTFLAGS='-D warnings'

if [[ "$enable_coverage" == "true" ]]; then
  output_dir=${output_dir:-$PWD/target/coverage/$(date +%Y-%m-%d_%H-%M-%S)}
  export CARGO_INCREMENTAL=0
  export RUSTFLAGS="$RUSTFLAGS -C instrument-coverage"
  export LLVM_PROFILE_FILE="$output_dir/kibi-%p-%m.profraw"
fi

cargo build \
  --target "$target" \
  --all-features \
  --locked \
  --verbose

if [[ "$run_tests" == "false" ]]; then
  exit 0
fi

cargo test \
  --target "$target" \
  --all-features \
  --locked \
  --no-fail-fast \
  --verbose

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