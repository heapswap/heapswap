#!/bin/bash

# Build the wasm bundle

cd crates/subfield
wasm-pack build --target web --dev --out-name index
rm -rf browser-test/src/wasm
mkdir -p browser-test/src/wasm
cp -r pkg/* browser-test/src/wasm
cd -
