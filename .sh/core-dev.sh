#!/bin/bash

# Build the wasm bundle
cd crates/core
wasm-pack build --target web --dev
rm -rf browser-test/src/wasm
mkdir -p browser-test/src/wasm
cp -r pkg/* browser-test/src/wasm
cd -

