use std::borrow::Borrow;
use std::borrow::BorrowMut;
use super::gc::{GcTraceable, GcNode};
use super::interned_string_map::InternedStringMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CowStringData {
    // data must last as long as the virtual machine
    // usage outside of vm execution is undefined
    // TODO: might be a better idea to use Rc?
    // however since all data structures in vmbindings
    // assume that they last as long as the Vm, it might be moot
    // to do this :?
    data: *const String,
}

#[derive(Clone, Debug, Eq)]
pub enum HaruStringData {
    CowString(CowStringData),
    String(String),
}

impl HaruStringData {
    fn is_cow(&self) -> bool {
        match self {
            HaruStringData::CowString(_) => true,
            _ => false,
        }
    }

    fn as_cow(&self) -> &String {
        match self {
            HaruStringData::CowString(s) => unsafe {
                &*s.data
            },
            _ => unreachable!(),
        }
    }
}

impl std::cmp::PartialEq for HaruStringData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HaruStringData::CowString(x), HaruStringData::CowString(y))
                => x == y,
            (x, y) => {
                let x = x.borrow() as &String;
                let y = y.borrow() as &String;
                x == y
            }
        }
    }
}

impl std::hash::Hash for HaruStringData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            HaruStringData::CowString(s) => (self.borrow() as &String).hash(state),
            HaruStringData::String(s) => s.hash(state)
        }
    }
}

impl std::borrow::Borrow<String> for HaruStringData {
    fn borrow(&self) -> &String {
        match &self {
            HaruStringData::CowString(s) => unsafe {
                &*s.data
            }
            HaruStringData::String(s) => {
                &s
            }
        }
    }
}

// expose
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HaruString {
    data: HaruStringData,
}

impl HaruString {

    pub fn new_cow(data: *const String) -> HaruString {
        HaruString {
            data: HaruStringData::CowString(CowStringData { data })
        }
    }

    pub fn is_cow(&self) -> bool {
        self.data.is_cow()
    }

}

impl std::borrow::Borrow<String> for HaruString {
    fn borrow(&self) -> &String {
        self.data.borrow()
    }
}
impl std::borrow::Borrow<str> for HaruString {
    fn borrow(&self) -> &str {
        (self.borrow() as &String).as_str()
    }
}
impl std::borrow::BorrowMut<String> for HaruString {
    fn borrow_mut(&mut self) -> &mut String {
        if self.data.is_cow() {
            self.data = HaruStringData::String(self.data.as_cow().clone());
        }
        match &mut self.data {
            HaruStringData::String(s) => s,
            _ => unreachable!(),
        }
    }
}

impl std::ops::Deref for HaruString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}
impl std::ops::DerefMut for HaruString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut()
    }
}

impl GcTraceable for HaruString {
    unsafe fn trace(&self, _manager: &mut Vec<*mut GcNode>) {}
}

// conversion
impl Into<HaruString> for String {

    fn into(self) -> HaruString {
        HaruString {
            data: HaruStringData::String(self)
        }
    }

}
impl From<&str> for HaruString {

    fn from(s: &str) -> Self {
        HaruString {
            data: HaruStringData::String(String::from(s))
        }
    }

}