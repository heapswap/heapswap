# Heapswap

⚠️ In-Progress ⚠️

This crate is an implementation of Heapswap's server-side engine.

Currently, it has:

- [x] A simple YJS/YRS server using Axum
- [x] A single-core embedding engine using [gte-small](https://huggingface.co/Supabase/gte-small)
  - Expects models to be in `src/models/gte-small` to have `model.onnx` and `tokenizer.json`
