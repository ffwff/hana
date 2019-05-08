use super::chmap::CHashMap;
use super::carray::CArray;
use super::cfunction::Function;
use super::cnativeval::NativeValue;
use super::vm::Vm;
use super::env::Env;

#[allow(unused_attributes)]
pub mod foreignc {

use std::ffi::CStr;
use std::ptr::{null, null_mut};
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
    Box::into_raw(Box::new(String::from(s)))
}

#[no_mangle]
pub unsafe extern "C" fn string_free(cstr: *mut String) {
    Box::from_raw(cstr);
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
    Box::into_raw(Box::new(left[to..to+1].to_string()))
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
    Box::into_raw(Box::new(Function::new(addr, nargs, env)))
}

#[no_mangle]
pub unsafe extern "C" fn function_free(fun: *mut Function) {
    Box::from_raw(fun);
}
// #endregion

// #region array
#[no_mangle]
pub unsafe extern "C" fn array_obj_malloc() -> *mut CArray<NativeValue> {
    Box::into_raw(Box::new(CArray::new()))
}
#[no_mangle]
pub unsafe extern "C" fn array_obj_malloc_n(n: usize) -> *mut CArray<NativeValue> {
    Box::into_raw(Box::new(CArray::reserve(n)))
}
#[no_mangle]
pub unsafe extern "C" fn array_obj_free(carray: *mut CArray<NativeValue>) {
    Box::from_raw(carray);
}
// #endregion

// #region env
#[no_mangle]
pub unsafe extern "C" fn env_malloc(parent: *mut Env, retip: u32, lexical_parent : *mut Env, nargs: u16) -> *mut Env {
    Box::into_raw(Box::new(Env::new(
        retip, Some(&*lexical_parent), nargs)))
}
#[no_mangle]
pub unsafe extern "C" fn env_init(selfptr: *mut Env, nslots: u16, vm: *mut Vm) {
    let env = &mut *selfptr;
    env.reserve(nslots);
}
#[no_mangle]
pub unsafe extern "C" fn env_free(selfptr: *mut Env) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "C" fn env_copy(selfptr: *mut Env) -> *mut Env {
    unimplemented!()
}

//
#[no_mangle]
pub unsafe extern "C" fn env_get(selfptr: *mut Env, nslots: u16) -> *mut Env {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "C" fn env_get_up(selfptr: *mut Env, up: u16, nslots: u16) -> *mut Env {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "C" fn env_set(selfptr: *mut Env, nslots: u16) -> *mut Env {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "C" fn env_set_up(selfptr: *mut Env, up: u16, nslots: u16) -> *mut Env {
    unimplemented!()
}

//
#[no_mangle]
pub unsafe extern "C" fn vm_leave_env(selfptr: *mut Vm) {
    let vm = &mut *selfptr;
    vm.leave_env();
}
// #endregion


}