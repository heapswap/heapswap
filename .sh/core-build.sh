#!/bin/bash

#export RUSTC_WRAPPER="sccache"

# Build the wasm bundle
cd crates/core
#wasm-pack build --target web --out-dir static
wasm-pack build --target web --dev

# Add the wasm bundle to the browser package
#rm -rf packages/browser/src/wasm
#mkdir packages/browser/src/wasm
#cp -r crates/browser/static/* packages/browser/src/wasm
