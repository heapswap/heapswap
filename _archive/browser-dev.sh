#!/bin/bash

#export RUSTC_WRAPPER="sccache"

# Build the wasm bundle
cd crates/browser
wasm-pack build --target web --dev
mkdir -p browser-test/src/wasm
cp -r pkg/* browser-test/src/wasm
cd -

# Add the wasm bundle to the browser package
#rm -rf packages/browser/src/wasm
#mkdir packages/browser/src/wasm
#cp -r crates/browser/static/* packages/browser/src/wasm
