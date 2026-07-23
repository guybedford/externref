# externref_t

Prototype of a real wasm `externref` lang type for Rust on
`wasm32-unknown-emscripten`.

The rustc fork adds `core::arch::wasm32::externref` behind
`feature(wasm_externref)`: an opaque host reference lowering to a true wasm
reference type in `extern "C"` signatures. Following clang's `__externref_t`
semantics, it is a bare-position-only type — legal only as a top-level
function parameter, return value or local — so JS values cross the boundary
directly and identity-preserved, with no handle tables or refcounting at the
FFI layer, and no possibility of a reference entering linear memory.

This crate layers on top:

* **`Externref`** — the owned, storable form: a slot in a wasm-native
  externref table declared via `global_asm!` (`feature(asm_experimental_arch)`),
  with single-instruction `table.get`/`table.set` accessors, a free-listed
  allocator in linear memory, `Clone`/`Drop` slot management, and `Debug` via
  JS `String()`. `from_raw`/`as_raw` are the only crossings to the bare lang
  type.
* **`ref_roundtrip`** — an exported function taking and returning `externref`,
  callable from JS directly on the wasm export.
* **`em_js_data!`** — emscripten `EM_JS` authored from Rust: emits the
  `__em_js__` data symbols that emcc extracts at link time to generate JS
  imports, used here for the test utilities (`testutil`).

## Setup

Requires an active emsdk (`emcc` on `PATH`, tested against 6.0.x) and node.

Build and link the rustc fork (branch `wasm-externref`):

```sh
git clone -b wasm-externref https://github.com/guybedford/rust
cd rust
./x build library --stage 1 --target wasm32-unknown-emscripten
rustup toolchain link externref-stage1 build/host/stage1
```

`rust-toolchain.toml` pins this project to `externref-stage1`, and
`.cargo/config.toml` targets `wasm32-unknown-emscripten` with a `node` runner.

## Running

```sh
cargo run    # minimal example
cargo test   # full suite, runs under node
```

Equivalently, without the config runner:

```sh
CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUNNER=node cargo test
```

## Notes

* Tests live in `tests/` (integration) only: EM_JS data symbols are
  internalized when a crate compiles as a bin, so the lib and bin targets set
  `test = false`.
* The free list is linear-memory-side because LLVM MC has no wasm-GC opcodes
  yet (`ref.i31`/`extern.convert_any`); freed slots are nulled
  (`ref.null_extern`) on `Drop` so dropped values are immediately
  GC-eligible.
