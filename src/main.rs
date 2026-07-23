//! Minimal usage example; the test suite in tests/e2e.rs is the full
//! demonstration.

use externref_t::Externref;
use externref_t::testutil::{make_ref, ref_json, refs_eq};

fn main() {
    // bare externref: Copy, locals/params/returns only
    let raw = make_ref();
    assert!(refs_eq(raw, raw));
    println!("raw json: {}", ref_json(raw));

    // owned Externref: table-backed, storable, Clone/Drop
    let owned = Externref::from_raw(raw);
    let clone = owned.clone();
    println!("owned debug: {:?}", owned);
    println!("clone same referent: {}", refs_eq(owned.as_raw(), clone.as_raw()));
}
