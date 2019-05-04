pub mod compiler {
    struct Scope {
        vars : Vec<String>
    }
    impl Scope {
        fn new() -> Scope {
            Scope { vars: Vec::with_capacity(std::u8::MAX as usize) }
        }
    }

    struct Identifier {
        slot : u8,
        relascope : u8
    }

    pub struct Compiler {
        scopes : Vec<Scope>
    }
    impl Compiler {
        fn get_local(&self, var : String) -> Identifier {
            let mut relascope : u8 = 0;
            for scope in self.scopes.iter().rev() {
                if let Some(slot) = scope.vars.iter().position(|&x| x == var) {
                    return Identifier{ slot: slot as u8, relascope: relascope };
                }
                relascope += 1;
            }
            panic!("can't get local")
        }

        fn set_local(&self, var : String) {
            if let Some(last) = self.scopes.last_mut() {
                last.vars.push(var);
            }
            panic!("scope is empty");
        }
    }
}