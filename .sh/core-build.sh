#!/bin/bash

# Build the wasm bundle
cd crates/core
wasm-pack build --target web --release
mkdir -p browser-test/src/wasm
cp -r pkg/* browser-test/src/wasm
cd -
