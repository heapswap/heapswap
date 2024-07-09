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
    -x "build"
else
  cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "data/*" \
    -i "src/data/*" \
    -i "src/schemas/*" \
    -c \
    -x "build --bin heapswap_$PACKAGE_NAME"
fi