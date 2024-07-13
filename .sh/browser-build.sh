#!/bin/bash

# Build the wasm bundle

cd crates/browser
#wasm-pack build --target web --out-dir static
wasm-pack build --target web --release
mkdir -p browser-test/src/wasm
cp -r pkg/* browser-test/src/wasm
cd -
