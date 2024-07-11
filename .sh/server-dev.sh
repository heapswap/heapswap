cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "data/*" \
    -i "src/data/*" \
    -i "src/schemas/*" \
    -i "crates/browser" \
    -c \
    -x "run --bin heapswap_server"