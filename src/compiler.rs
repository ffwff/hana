use crate::vm::Vm;

// private
struct Scope {
    vars : Vec<String>
}
impl Scope {
    fn new() -> Scope {
        Scope { vars: Vec::with_capacity(std::u8::MAX as usize) }
    }
}

// public
pub struct Identifier {
    pub slot : u8,
    pub relascope : u8
}
pub struct Compiler {
    scopes : Vec<Scope>,
    pub vm : Vm
}
impl Compiler {
    pub fn new() -> Compiler {
        Compiler{
            scopes: Vec::new(),
            vm: Vm::new()
        }
    }

    pub fn get_local(&self, var : String) -> Identifier {
        let mut relascope : u8 = 0;
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.vars.iter().position(|x| *x == var) {
                return Identifier{ slot: slot as u8, relascope: relascope };
            }
            relascope += 1;
        }
        panic!("can't get local")
    }

    pub fn set_local(&mut self, var : String) {
        if let Some(last) = self.scopes.last_mut() {
            last.vars.push(var);
        }
        panic!("scope is empty");
    }
}
