cargo watch \
    -i "target/*" \
    -i "dist/*" \
    -i "crates/subfield" \
    -i "crates/heapswap/_subfield_store" \
    -c \
    -x "run --bin heapswap -- --nocapture"