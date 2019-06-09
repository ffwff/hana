use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::rc::Rc;
use super::gc::{GcTraceable, GcNode};
use super::interned_string_map::InternedStringMap;

#[derive(Clone, Debug)]
pub struct CowStringData {
    data: Rc<String>,
}

impl std::cmp::PartialEq for CowStringData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.data.as_ref(), other.data.as_ref())
    }
}

impl std::cmp::Eq for CowStringData {}

//

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
            HaruStringData::CowString(s) => &*s.data,
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
            HaruStringData::CowString(s) => self.as_cow(),
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

    pub fn new_cow(data: Rc<String>) -> HaruString {
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