//! `Externref`: an owned JS reference held in a wasm-native externref table.
//!
//! The table and its single-instruction accessors are defined in MC assembly
//! via `global_asm!`, typed at the `extern "C"` boundary by the externref
//! lang type. No JS in the hot path; the free list lives in linear memory
//! (LLVM MC has no wasm-GC opcodes yet, so no i31 links in-table).

use core::arch::wasm32::externref;
use std::arch::global_asm;
use std::fmt;
use std::sync::Mutex;

global_asm!(
    ".section .data.RUST_REF_TABLE,\"\",@",
    ".tabletype RUST_REF_TABLE, externref",
    "RUST_REF_TABLE:",
    //
    ".globl rust_table_get",
    ".section .text.rust_table_get,\"\",@",
    "rust_table_get:",
    ".functype rust_table_get (i32) -> (externref)",
    "local.get 0",
    "table.get RUST_REF_TABLE",
    "end_function",
    //
    ".globl rust_table_set",
    ".section .text.rust_table_set,\"\",@",
    "rust_table_set:",
    ".functype rust_table_set (i32, externref) -> ()",
    "local.get 0",
    "local.get 1",
    "table.set RUST_REF_TABLE",
    "end_function",
    //
    ".globl rust_table_remove",
    ".section .text.rust_table_remove,\"\",@",
    "rust_table_remove:",
    ".functype rust_table_remove (i32) -> ()",
    "local.get 0",
    "ref.null_extern",
    "table.set RUST_REF_TABLE",
    "end_function",
    //
    ".globl rust_table_grow",
    ".section .text.rust_table_grow,\"\",@",
    "rust_table_grow:",
    ".functype rust_table_grow (i32) -> (i32)",
    "ref.null_extern",
    "local.get 0",
    "table.grow RUST_REF_TABLE",
    "end_function",
    //
    ".globl rust_table_size",
    ".section .text.rust_table_size,\"\",@",
    "rust_table_size:",
    ".functype rust_table_size () -> (i32)",
    "table.size RUST_REF_TABLE",
    "end_function",
);

unsafe extern "C" {
    fn rust_table_get(i: u32) -> externref;
    fn rust_table_set(i: u32, v: externref);
    fn rust_table_remove(i: u32);
    fn rust_table_grow(delta: u32) -> i32;
    fn rust_table_size() -> u32;
}

crate::em_js_data!(
    __em_js__ref_to_string,
    "(v, outLen)<::>{ const bytes = new TextEncoder().encode(String(v)); const ptr = _malloc(bytes.length); HEAPU8.set(bytes, ptr); HEAPU32[outLen >>> 2] = bytes.length; return ptr; }"
);

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn ref_to_string(v: externref, out_len: *mut usize) -> *mut u8;
}

unsafe extern "C" {
    fn free(ptr: *mut core::ffi::c_void);
}

/// (free list, next never-used index)
static STATE: Mutex<(Vec<u32>, u32)> = Mutex::new((Vec::new(), 0));

/// An owned JS reference, held live in the externref table for the lifetime
/// of the value. `from_raw`/`as_raw` cross to the bare `externref` lang type
/// at the boundary; everything in between is ordinary storable Rust data.
pub struct Externref(u32);

impl Externref {
    pub fn from_raw(v: externref) -> Self {
        let mut state = STATE.lock().unwrap();
        let (free, next) = &mut *state;
        let i = free.pop().unwrap_or_else(|| {
            let i = *next;
            *next += 1;
            unsafe {
                if i >= rust_table_size() {
                    rust_table_grow(rust_table_size().max(4));
                }
            }
            i
        });
        unsafe { rust_table_set(i, v) };
        Externref(i)
    }

    pub fn as_raw(&self) -> externref {
        unsafe { rust_table_get(self.0) }
    }
}

impl Clone for Externref {
    fn clone(&self) -> Self {
        Externref::from_raw(self.as_raw())
    }
}

impl Drop for Externref {
    fn drop(&mut self) {
        unsafe { rust_table_remove(self.0) };
        STATE.lock().unwrap().0.push(self.0);
    }
}

impl fmt::Debug for Externref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::hint::black_box(&__em_js__ref_to_string);
        let mut len: usize = 0;
        unsafe {
            let ptr = ref_to_string(self.as_raw(), &mut len);
            let s = String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len));
            let r = f.write_str(&s);
            free(ptr as *mut core::ffi::c_void);
            r
        }
    }
}

/// Current size of the backing table (test/diagnostic visibility).
pub fn table_size() -> u32 {
    unsafe { rust_table_size() }
}
