//! externref_t: prototype of a real wasm `externref` lang type for Rust on
//! wasm32-unknown-emscripten.
//!
//! Requires the `wasm-externref` rustc fork, which provides
//! `core::arch::wasm32::externref`: an opaque host reference usable only in
//! bare function parameter, return and local positions, lowering to a real
//! wasm reference type in `extern "C"` signatures.

#![feature(wasm_externref, asm_experimental_arch)]

pub mod asmtable;
pub mod testutil;

pub use asmtable::Externref;

use core::arch::wasm32::externref;

#[doc(hidden)]
pub const fn __em_js_len(s: &str) -> usize {
    s.len() + 1
}

#[doc(hidden)]
pub const fn __em_js_bytes<const N: usize>(s: &str) -> [u8; N] {
    let b = s.as_bytes();
    let mut out = [0u8; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i];
        i += 1;
    }
    out
}

/// Emit an emscripten EM_JS-convention JS function from Rust.
///
/// `$sym` must be `__em_js__<name>` (or `__em_js____asyncjs__<name>` for an
/// import that should receive `WebAssembly.Suspending` treatment under
/// `-sJSPI`), and `$body` the `"(args)<::>{ body }"` string. Must be used in
/// a `lib` crate (rustc internalizes `#[no_mangle]` statics when compiling a
/// `bin`, dropping the data export emscripten's metadata extraction keys
/// on), and the static must be referenced from linked code via `black_box`
/// so the archive member is pulled in. Never add `#[link_section]`: rustc
/// emits wasm custom sections, breaking extraction.
#[doc(hidden)]
#[macro_export]
macro_rules! em_js_data {
    ($sym:ident, $body:expr) => {
        #[unsafe(no_mangle)]
        #[used]
        pub static $sym: [u8; $crate::__em_js_len($body)] =
            $crate::__em_js_bytes::<{ $crate::__em_js_len($body) }>($body);
    };
}

/// Exported to JS (`-sEXPORTED_FUNCTIONS=..,_ref_roundtrip`): takes any JS
/// value and returns it identity-preserved. JS calls `_ref_roundtrip(v)`
/// directly on the wasm export; the value crosses as a bare externref.
#[unsafe(no_mangle)]
pub extern "C" fn ref_roundtrip(v: externref) -> externref {
    v
}
