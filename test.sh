#!/bin/bash
set -euo pipefail

# TODO: we should be able to build without builder
cargo check --no-default-features --features "builder"
cargo check
cargo check --no-default-features --features "builder smtp-transport"
cargo check --no-default-features --features "builder smtp-transport native-tls"
cargo check --no-default-features --features "builder smtp-transport rustls-tls"
cargo check --no-default-features --features "builder smtp-transport native-tls rustls-tls"
cargo check --no-default-features --features "builder smtp-transport tokio02"
cargo check --no-default-features --features "builder smtp-transport tokio02 tokio02-native-tls"
cargo check --no-default-features --features "builder smtp-transport tokio02 tokio02-rustls-tls"
cargo check --no-default-features --features "builder smtp-transport tokio02 tokio02-native-tls tokio02-rustls-tls"
cargo check --all-features

cargo check --tests --examples --no-default-features --features "builder"
cargo check --tests --examples
cargo check --tests --examples --no-default-features --features "builder smtp-transport tokio02"
cargo check --tests --examples --no-default-features --features "builder smtp-transport tokio02 tokio02-native-tls"
cargo check --tests --examples --no-default-features --features "builder smtp-transport tokio02 tokio02-rustls-tls"
cargo check --tests --examples --no-default-features --features "builder smtp-transport tokio02 tokio02-native-tls tokio02-rustls-tls"
cargo check --tests --examples --all-features

cargo test --lib --all-features
