extern crate haru;

#[cfg(test)]
pub mod gc_tests {

    use haru::gc::GcManager;

    // #region simple
    #[test]
    pub fn simple_int() {
        let mut manager = GcManager::new();
        let i = manager.malloc_safe(10, GcManager::no_finalizer);
        assert_eq!(*i, 10);
    }

    #[test]
    pub fn simple_string() {
        let mut manager = GcManager::new();
        fn free_str(selfptr: *mut u8) {
            unsafe { GcManager::drop::<String>(selfptr); }
        }
        let i = manager.malloc_safe(String::from("Hello World"), free_str);
        //assert_eq!(*i, String::from("Hello World"));
    }
    // #endregion

}