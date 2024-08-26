#!/bin/bash

PACKAGE_NAME=$1

if [ -z "$PACKAGE_NAME" ]
then
  cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "data/*" \
    -i "src/data/*" \
    -i "src/schemas/*" \
    -c \
    -x "run"
else
  cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "data/*" \
    -i "src/data/*" \
    -i "src/schemas/*" \
    -i "crates/browser/browser-test" \
    -c \
    -x "run --bin $PACKAGE_NAME"
fi