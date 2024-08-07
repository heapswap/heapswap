#!/bin/bash

PACKAGE_NAME=$1
shift 

if [ -z "$PACKAGE_NAME" ]
then
  cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "data/*" \
    -i "src/data/*" \
    -i "src/schemas/*" \
    -c \
    -x "test -- --nocapture $@"
else
  cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "data/*" \
    -i "src/data/*" \
    -i "src/schemas/*" \
    -c \
    -x "test -p heapswap_$PACKAGE_NAME $@ -- --nocapture"
fi