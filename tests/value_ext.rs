extern crate haru;
use haru::vmbindings::value::Value;
use haru::vmbindings::cnativeval::NativeValue;
use haru::vmbindings::record::Record;

pub trait ValueExt {
    fn int(&self) -> i64;
    fn float(&self) -> f64;
    fn string(&self) -> &'static String;
    fn array(&self) -> &'static Vec<NativeValue>;
    fn record(&self) -> &'static Record;
}

impl ValueExt for Value {
    // #region coerce value to type
    #[cfg_attr(tarpaulin, skip)]
    fn int(&self) -> i64 {
        match self {
            Value::Int(s) => *s,
            _ => {
                panic!("Expected integer");
            }
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn float(&self) -> f64 {
        match self {
            Value::Float(s) => *s,
            _ => {
                panic!("Expected float");
            }
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn string(&self) -> &'static String {
        match self {
            Value::Str(s) => unsafe { &*s.to_raw() },
            _ => {
                panic!("Expected string");
            }
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn array(&self) -> &'static Vec<NativeValue> {
        match self {
            Value::Array(s) => unsafe { &*s.to_raw() },
            _ => {
                panic!("Expected array");
            }
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn record(&self) -> &'static Record {
        match self {
            Value::Record(rec) => unsafe { &*rec.to_raw() },
            _ => {
                panic!("Expected record");
            }
        }
    }
    // #endregion
}

// TODO FIXME
pub trait UnwrapUnsafe {
    type Output;
    fn unwraps(&self) -> Self::Output;
}

impl UnwrapUnsafe for NativeValue {
    type Output = Value;
    fn unwraps(&self) -> Value {
        unsafe{ self.unwrap() }
    }
}