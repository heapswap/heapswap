cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "data/*" \
    -i "src/data/*" \
    # -i "src/schemas/*" \
    # -i "crates/browser" \
    # -i "crates/core" \
    # -i "crates/protos" \
    -i "crates/subfield" \
    -c \
    -x "run --bin heapswap"