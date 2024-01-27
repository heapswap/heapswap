# for unstable features 
# rustup update -- nightly
# rustup default nightly

# # for building faiss
# sudo apt install libblas-dev
# sudo apt-get install liblapack-dev

# # for building rocksdb
# sudo apt-get install libgflags-dev libsnappy-dev zlib1g-dev libbz2-dev liblz4-dev libzstd-dev

sudo apt install -y pkg-config libssl-dev clang jq

# add rust and json formatter
cat << 'EOF' > .git/hooks/pre-commit
#!/bin/bash

# Run cargo fmt on staged Rust files
rust_files=$(git diff --cached --name-only --diff-filter=ACM ".rs")
if [[ -n "$rust_files" ]]; then
    echo "Running cargo fmt on staged Rust files:"
    echo "$rust_files"
    cargo fmt -- "$rust_files"
    git add $rust_files
fi

# Format staged JSON files
json_files=$(git diff --cached --name-only --diff-filter=ACM ".json")
if [[ -n "$json_files" ]]; then
    echo "Running jq on staged JSON files:"
    echo "$json_files"
    for file in $json_files; do
        jq . "$file" > "$file.formatted"
        mv "$file.formatted" "$file"
        git add "$file"
    done
fi
EOF