#! /bin/sh

set -eo pipefail

TOOLCHAIN=$1

if [ -z "$TOOLCHAIN" ]; then
    echo "Usage: ./run-tests.sh TOOLCHAIN"
    exit 1
fi

apt update
apt install -y curl build-essential openssl libssl-dev

if [ ! -f ~/.cargo/bin/rustup ]; then
    curl https://sh.rustup.rs -sSf | bash -s -- -y --default-toolchain=none
fi

~/.cargo/bin/rustup install $TOOLCHAIN
~/.cargo/bin/rustup default $TOOLCHAIN

~/.cargo/bin/cargo test

