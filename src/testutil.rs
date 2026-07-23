//! EM_JS-based helpers for tests and examples. These live in the lib crate
//! because EM_JS data symbols don't survive compilation in bin crates (which
//! is what integration test crates are).

use core::arch::wasm32::externref;

crate::em_js_data!(
    __em_js__test_make_ref,
    "()<::>{ return { hello: 'externref', life: 42 }; }"
);
crate::em_js_data!(
    __em_js__test_refs_eq,
    "(a, b)<::>{ return a === b ? 1 : 0; }"
);
crate::em_js_data!(
    __em_js__test_ref_json,
    "(v, outLen)<::>{ const bytes = new TextEncoder().encode(JSON.stringify(v)); const ptr = _malloc(bytes.length); HEAPU8.set(bytes, ptr); HEAPU32[outLen >>> 2] = bytes.length; return ptr; }"
);

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn test_make_ref() -> externref;
    fn test_refs_eq(a: externref, b: externref) -> i32;
    fn test_ref_json(v: externref, out_len: *mut usize) -> *mut u8;
}

unsafe extern "C" {
    fn free(ptr: *mut core::ffi::c_void);
}

#[inline(never)]
fn anchor() {
    std::hint::black_box(&__em_js__test_make_ref);
    std::hint::black_box(&__em_js__test_refs_eq);
    std::hint::black_box(&__em_js__test_ref_json);
}

/// A fresh JS test object as a bare externref.
pub fn make_ref() -> externref {
    anchor();
    unsafe { test_make_ref() }
}

/// JS `===` over bare externrefs.
pub fn refs_eq(a: externref, b: externref) -> bool {
    anchor();
    unsafe { test_refs_eq(a, b) == 1 }
}

/// `JSON.stringify` over a bare externref.
pub fn ref_json(v: externref) -> String {
    anchor();
    let mut len: usize = 0;
    unsafe {
        let ptr = test_ref_json(v, &mut len);
        let s = String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned();
        free(ptr as *mut core::ffi::c_void);
        s
    }
}
