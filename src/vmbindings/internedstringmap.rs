//! Provides interned string map

const MAX_LENGTH : usize = std::u16::MAX as usize;

pub struct InternedStringMap {
    data: Vec<String>,
}

impl InternedStringMap {

    pub fn new() -> InternedStringMap {
        InternedStringMap {
            data: Vec::new()
        }
    }

    pub fn get_or_insert<T: AsRef<str>>(&mut self, s: &T) -> Option<u16> {
        let s : &str = s.as_ref();
        // only intern string in this length range to save up memory
        if (2..20).contains(&s.len()) {
            return None;
        }
        let it = self.data.iter()
                .enumerate()
                .filter(|(_, key)| key.as_str() == s)
                .next();
        if let Some((idx, _)) = it {
            Some(idx as u16)
        } else {
            self.data.push(String::from(s));
            assert!(self.data.len() < MAX_LENGTH);
            Some((self.data.len() - 1) as u16)
        }
    }

    pub unsafe fn get_unchecked(&self, idx: u16) -> &String {
        self.data.get_unchecked(idx as usize)
    }

}