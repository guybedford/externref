# externref_t

Prototype of a real wasm `externref` lang type for Rust on
`wasm32-unknown-emscripten`, then extending that with an addressable
first-class `Externref` wrapper using `global_asm!` operations for the
externref storage and retrieval.

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

## Example

```rust
#![feature(wasm_externref)]

use core::arch::wasm32::externref;
use externref_t::Externref;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn get_document_raw() -> externref;
}

fn get_document() -> Externref {
    Externref::from_raw(unsafe { get_document_raw() })
}

pub fn main() {
    let mut docs = Vec::new();
    let doc = get_document();

    // can clone Externref and use in generic types
    docs.push(doc.clone());
}
```

## Running the Tests

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
