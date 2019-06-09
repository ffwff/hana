use std::borrow::Borrow;
use std::borrow::BorrowMut;
use super::gc::{GcTraceable, GcNode};
use super::interned_string_map::InternedStringMap;

pub struct CowStringData {
    idx: u16,
    // map must last as long as the virtual machine
    // usage outside of vm execution is undefined
    // TODO: might be a better idea to use Rc?
    // however since all data structures in vmbindings
    // assume that they last as long as the Vm, it might be moot
    // to do this :?
    map: *const InternedStringMap,
}

impl std::fmt::Debug for CowStringData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CowStringData {{}}")
    }
}

#[derive(Debug)]
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
                (*s.map).get(s.idx).unwrap()
            },
            _ => unreachable!(),
        }
    }
}

// expose
#[derive(Debug)]
pub struct HaruString {
    data: HaruStringData,
}

impl HaruString {

    pub fn new_cow(idx: u16, map: *const InternedStringMap) -> HaruString {
        eprintln!("idx: {}", idx);
        HaruString {
            data: HaruStringData::CowString(CowStringData { idx, map })
        }
    }

    pub fn is_cow(&self) -> bool {
        self.data.is_cow()
    }

}

impl std::borrow::Borrow<String> for HaruString {
    fn borrow(&self) -> &String {
        match &self.data {
            HaruStringData::CowString(s) => unsafe {
                (*s.map).get(s.idx).unwrap()
            }
            HaruStringData::String(s) => {
                &s
            }
        }
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