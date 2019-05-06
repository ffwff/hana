use super::chmap::CHashMap;
use super::cnativeval::NativeValue;

#[allow(unused_attributes)]
pub mod foreignc {

use std::ptr::null;
use std::ffi::CStr;
use super::CHashMap;
use super::NativeValue;

// #region hmap
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

// #endregion

// #region dict
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
pub extern "C" fn dict_set_cptr() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn dict_get() {
    unimplemented!()
}
#[no_mangle]
pub extern "C" fn dict_get_cptr() {
    unimplemented!()
}

// values
#[no_mangle]
pub extern "C" fn value_dict() {
    unimplemented!()
}

// #endregion

// #region string
#[no_mangle]
pub unsafe extern "C" fn string_malloc(cstr: *mut libc::c_char) -> *mut String {
    let s = CStr::from_ptr(cstr).to_str().unwrap();
    Box::into_raw(Box::new(String::from(s)))
}

#[no_mangle]
pub unsafe extern "C" fn string_append(cleft: *const String, cright: *const String) -> *mut String {
    let left : &'static String = &*cleft;
    let right : &'static String = &*cright;
    let mut newleft = left.clone();
    newleft += right;
    Box::into_raw(Box::new(newleft))
}

#[no_mangle]
pub unsafe extern "C" fn string_repeat(cleft: *const String, n : i64) -> *mut String {
    let left : &'static String = &*cleft;
    Box::into_raw(Box::new(left.repeat(n as usize)))
}

#[no_mangle]
pub extern "C" fn string_cmp() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn string_at() {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "C" fn string_is_empty(s: *const String) -> bool {
    let left : &'static String = &*s;
    left.len() == 0
}
// #endregion

}