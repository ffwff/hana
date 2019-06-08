//! Provides a hashmap of String-NativeValue

use super::nativeval::NativeValue;
use std::collections::HashMap;

/// A hashmap of String-NativeValue
pub type HaruHashMap = HashMap<String, NativeValue>;
