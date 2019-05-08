use super::chmap::CHashMap;
use super::carray::CArray;
use super::cfunction::{Env, Function};
use super::cnativeval::NativeValue;
use super::gc::{malloc, free, drop};

#[allow(unused_attributes)]
pub mod foreignc {

use std::ptr::null;
use std::ffi::CStr;
use super::*;

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
pub unsafe extern "C" fn hmap_get(chm: *const CHashMap, ckey: *const libc::c_char) -> *const NativeValue {
    let key = String::from(CStr::from_ptr(ckey).to_str().clone().unwrap());
    let hm = &*chm;
    if let Some(val) = hm.get(&key) {
        val
    } else {
        null()
    }
}
#[no_mangle]
pub unsafe extern "C" fn hmap_get_str(chm: *const CHashMap, ckey: *const String) -> *const NativeValue {
    let key = &*ckey;
    let hm = &*chm;
    if let Some(val) = hm.get(key) {
        val
    } else {
        null()
    }
}

#[no_mangle]
pub unsafe extern "C" fn hmap_set(chm: *mut CHashMap, ckey: *const libc::c_char, cval: *const NativeValue) {
    let key = String::from(CStr::from_ptr(ckey).to_str().clone().unwrap());
    let val = (*cval).clone();
    let hm = &mut *chm;
    hm.insert(key, val);
}
#[no_mangle]
pub unsafe extern "C" fn hmap_set_str(chm: *mut CHashMap, ckey: *const  String, cval: *const NativeValue) {
    let key = (&*ckey).clone();
    let val = (*cval).clone();
    let hm = &mut *chm;
    hm.insert(key, val);
}

// #endregion

// #region string
#[no_mangle]
pub unsafe extern "C" fn string_malloc(cstr: *mut libc::c_char) -> *mut String {
    let s = CStr::from_ptr(cstr).to_str().unwrap();
    malloc(String::from(s), |ptr| drop::<String>(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn string_free(cstr: *mut String) {
    //free(cstr);
}

#[no_mangle]
pub unsafe extern "C" fn string_append(cleft: *const String, cright: *const String) -> *mut String {
    let left : &'static String = &*cleft;
    let right : &'static String = &*cright;
    let mut newleft = left.clone();
    newleft += right;
    malloc(newleft, |ptr| drop::<String>(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn string_repeat(cleft: *const String, n : i64) -> *mut String {
    let left : &'static String = &*cleft;
    malloc(left.repeat(n as usize), |ptr| drop::<String>(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn string_cmp(cleft: *const String, cright: *const String) -> i64 {
    let left : &'static String = &*cleft;
    let right : &'static String = &*cright;
    if left == right     {  0 }
    else if left < right { -1 }
    else                 {  1 }
}

#[no_mangle]
pub unsafe extern "C" fn string_at(cleft: *const String, idx : i64) -> *mut String {
    let left : &'static String = &*cleft;
    let to = idx as usize;
    malloc(left[to..to+1].to_string(), |ptr| drop::<String>(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn string_is_empty(s: *const String) -> bool {
    let left : &'static String = &*s;
    left.len() == 0
}
// #endregion

// #region function
#[no_mangle]
pub unsafe extern "C" fn function_malloc(addr: u32, nargs: u16, env: *const Env) -> *mut Function {
    malloc(Function::new(addr, nargs, env), |ptr| drop::<Function>(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn function_free(fun: *mut Function) {
    //Box::from_raw(fun);
}
// #endregion

// #region array
#[no_mangle]
pub unsafe extern "C" fn array_obj_malloc() -> *mut CArray<NativeValue> {
    malloc(CArray::new(), |ptr| {
        let array = &mut *(ptr as *mut CArray<NativeValue>);
        array.drop();
    })
}
#[no_mangle]
pub unsafe extern "C" fn array_obj_malloc_n(n: usize) -> *mut CArray<NativeValue> {
    malloc(CArray::reserve(n), |ptr| {
        let array = &mut *(ptr as *mut CArray<NativeValue>);
        array.drop();
    })
}
#[no_mangle]
pub unsafe extern "C" fn array_obj_free(carray: *mut CArray<NativeValue>) {
    //Box::from_raw(carray);
}
// #endregion

}