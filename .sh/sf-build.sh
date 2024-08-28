#!/bin/bash

# Build the wasm bundle

cd crates/subfield
wasm-pack build --target web --out-name index --release --features "client"
mkdir -p browser-test/src/wasm
cp -r pkg/* browser-test/src/wasm
cd -
