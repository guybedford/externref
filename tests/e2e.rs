#![feature(wasm_externref)]

use externref_t::testutil::{make_ref, ref_json, refs_eq};
use externref_t::{Externref, asmtable, ref_roundtrip};

#[test]
fn bare_externref_copy_and_identity() {
    let r = make_ref();
    let copy = r;
    assert!(refs_eq(r, copy));
    assert!(!refs_eq(make_ref(), make_ref()));
}

#[test]
fn bare_externref_value() {
    assert_eq!(ref_json(make_ref()), r#"{"hello":"externref","life":42}"#);
}

#[test]
fn exported_roundtrip_identity() {
    let r = make_ref();
    assert!(refs_eq(ref_roundtrip(r), r));
}

#[test]
fn externref_from_raw_as_raw() {
    let a = Externref::from_raw(make_ref());
    assert!(refs_eq(a.as_raw(), a.as_raw()));
    assert_eq!(ref_json(a.as_raw()), r#"{"hello":"externref","life":42}"#);
}

#[test]
fn externref_clone_shares_referent() {
    let a = Externref::from_raw(make_ref());
    let b = a.clone();
    assert!(refs_eq(a.as_raw(), b.as_raw()));
}

#[test]
fn externref_debug() {
    let a = Externref::from_raw(make_ref());
    assert_eq!(format!("{a:?}"), "[object Object]");
}

#[test]
fn externref_slot_reuse_and_growth() {
    let a = Externref::from_raw(make_ref());
    let size = asmtable::table_size();
    drop(a);
    let _b = Externref::from_raw(make_ref());
    assert_eq!(asmtable::table_size(), size, "freed slot should be reused");

    let held: Vec<Externref> = (0..64).map(|_| Externref::from_raw(make_ref())).collect();
    assert!(asmtable::table_size() >= 64);
    assert!(!refs_eq(held[0].as_raw(), held[1].as_raw()));
}

#[test]
fn externref_storable() {
    struct Holder {
        r: Externref,
    }
    let h = Holder { r: Externref::from_raw(make_ref()) };
    let v = vec![h.r.clone(), h.r.clone()];
    assert!(refs_eq(v[0].as_raw(), v[1].as_raw()));
}
