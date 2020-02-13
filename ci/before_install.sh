#!/usr/bin/env bash

set -euxo pipefail

if [ "$TRAVIS_OS_NAME" == linux ] && [ "$HOST" != "${TARGET:-$HOST}"  ]; then
  case "$TARGET" in
  'i686-unknown-linux-gnu')
    sudo apt-get update
    sudo apt-get install -y gcc-multilib
    ;;
  esac

  rustup target add "$TARGET";
fi

rustup self update