#!/bin/bash

# musl target must be installed:
#  rustup target add x86_64-unknown-linux-musl

cargo build --release --target=x86_64-unknown-linux-musl
