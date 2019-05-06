#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Nil,
    Int(i64),
    Float(f64),
    NativeFn,
    Fn,
    Str(&'static String),
    Dict,
    Array,
    NativeObj
}

impl Value {

    pub fn string(&self) -> &'static String {
        match self {
            Value::Str(s) => s,
            _ => { panic!("Expected string"); }
        }
    }

}