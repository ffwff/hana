//! Provides a hashmap of String-NativeValue

use std::collections::HashMap;
use super::cnativeval::NativeValue;

/// A hashmap of String-NativeValue
pub type CHashMap = HashMap<String, NativeValue>;