use super::chmap::CHashMap;
use super::cnativeval::NativeValue;
use super::value::Value;

#[allow(unused_attributes)]
pub mod foreignc {

use std::ptr::null;
use std::ffi::{CStr, CString};
use super::CHashMap;
use super::NativeValue;
use super::Value;

// hmap
#[no_mangle]
pub extern "C" fn hmap_malloc() -> *mut CHashMap {
    Box::into_raw(Box::new(CHashMap::new()))
}

#[no_mangle]
pub unsafe extern "C" fn hmap_free(hm: *mut CHashMap) {
    if hm.is_null() { return; }
    Box::from_raw(hm);
}

#[no_mangle]
pub unsafe extern "C" fn hmap_get(chm: *mut CHashMap, ckey: *mut libc::c_char) -> *const NativeValue {
    let key = String::from(CStr::from_ptr(ckey).to_str().clone().unwrap());
    let hm = &*chm;
    if let Some(val) = hm.get(&key) {
        val
    } else {
        null()
    }
}

#[no_mangle]
pub unsafe extern "C" fn hmap_set(chm: *mut CHashMap, ckey: *mut libc::c_char, cval: *const NativeValue) {
    let key = String::from(CStr::from_ptr(ckey).to_str().clone().unwrap());
    let val = (*cval).clone();
    let hm = &mut *chm;
    hm.insert(key, val);
}

// dict
#[no_mangle]
pub extern "C" fn dict_init() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn dict_free() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn dict_set() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn dict_get() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn dict_get_prototype() {
    unimplemented!()
}

// values
// dicts
#[no_mangle]
pub extern "C" fn value_dict() {
    unimplemented!()
}

}