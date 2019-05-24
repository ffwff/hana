use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::vm::Vm;
use crate::vm::VmOpcode;
use crate::vmbindings::carray::CArray;

// private
struct Scope {
    vars : Vec<String>
}
impl Scope {
    fn new() -> Scope {
        Scope { vars: Vec::new() }
    }
}

struct LoopStatement {
    pub fill_continue: Vec<usize>,
    pub fill_break: Vec<usize>
}

// public
pub type ArrayIndexRange = (usize, usize);
pub struct SourceMap {
    pub file: ArrayIndexRange,
    pub bytecode: ArrayIndexRange,
    pub fileno: usize,
}

macro_rules! bvm {
    ($c:ident) => ($c.vm.borrow_mut());
}

pub struct Compiler {
    scopes : Vec<Scope>,
    loop_stmts : Vec<LoopStatement>,
    pub smap: Vec<SourceMap>,
    pub files: Vec<String>,
    pub modules_loaded: std::collections::HashSet<std::path::PathBuf>,
    pub symbol: HashMap<usize, String>,
    pub sources: Vec<String>,
    pub vm : Rc<RefCell<Vm>>
}
impl Compiler {

    pub fn new() -> Compiler {
        Compiler{
            scopes: Vec::new(),
            loop_stmts: Vec::new(),
            smap: Vec::new(),
            files: Vec::new(),
            modules_loaded: std::collections::HashSet::new(),
            symbol: HashMap::new(),
            sources: Vec::new(),
            vm: Vm::new()
        }
    }

    // constructor for execution ctx
    pub fn new_append_vm(vm: &mut Vm) -> Compiler {
        use std::ptr::null_mut;
        use crate::vmbindings::vmerror::VmError;
        Compiler{
            scopes: Vec::new(),
            loop_stmts: Vec::new(),
            smap: Vec::new(),
            files: Vec::new(),
            modules_loaded: std::collections::HashSet::new(),
            symbol: HashMap::new(),
            sources: Vec::new(),
            vm: Rc::new(RefCell::new({
                let mut vm_ = Vm::new_nil();
                vm_.code = vm.code.deref();
                vm_
            }))
        }
    }

    pub fn deref_vm_code(mut self) -> CArray<VmOpcode> {
        unimplemented!()
        // bvm!(self).try_unwrap().unwrap().code.deref()
    }

    // scopes
    pub fn is_in_function(&self) -> bool {
        !self.scopes.is_empty()
    }

    // local
    fn get_local(&self, var : &String) -> Option<(u16, u16)> {
        let mut relascope : u16 = 0;
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.vars.iter().position(|x| *x == *var) {
                return Some((slot as u16, relascope));
            }
            relascope += 1;
        }
        None
    }

    pub fn set_local(&mut self, var : String) -> Option<(u16, u16)> {
        if let Some(last) = self.scopes.last_mut() {
            last.vars.push(var);
            let idx = last.vars.len()-1;
            return Some((idx as u16, 0));
        }
        None
    }

    // emit set var
    pub fn emit_set_var(&mut self, var : String) {
        if var.starts_with("$") || self.scopes.len() == 0 {
            // set global
            bvm!(self).code.push(VmOpcode::OP_SET_GLOBAL);
            bvm!(self).cpushs(if var.starts_with("$") { &var[1..] }
                           else { var.as_str() });
        } else if let Some(local) = self.get_local(&var) {
            // set existing local
            let mut slot = local.0;
            let relascope = local.1;
            if relascope != 0 {
                let local = self.set_local(var.clone()).unwrap();
                slot = local.0;
            }
            bvm!(self).code.push(VmOpcode::OP_SET_LOCAL);
            bvm!(self).cpush16(slot);
        } else {
            let local = self.set_local(var.clone()).unwrap();
            let slot = local.0;
            bvm!(self).code.push(VmOpcode::OP_SET_LOCAL);
            bvm!(self).cpush16(slot);
        }
    }
    pub fn emit_set_var_fn(&mut self, var : String) {
        if var.starts_with("$") || self.scopes.len() == 0 {
            // set global
            bvm!(self).code.push(VmOpcode::OP_SET_GLOBAL);
            bvm!(self).cpushs(if var.starts_with("$") { &var[1..] }
                           else { var.as_str() });
        } else if let Some(local) = self.get_local(&var) {
            // set existing local
            let mut slot = local.0;
            let relascope = local.1;
            if relascope != 0 {
                let local = self.set_local(var.clone()).unwrap();
                slot = local.0;
            }
            bvm!(self).code.push(VmOpcode::OP_SET_LOCAL_FUNCTION_DEF);
            bvm!(self).cpush16(slot);
        } else {
            let local = self.set_local(var.clone()).unwrap();
            let slot = local.0;
            bvm!(self).code.push(VmOpcode::OP_SET_LOCAL_FUNCTION_DEF);
            bvm!(self).cpush16(slot);
        }
    }

    pub fn emit_get_var(&mut self, var : String) {
        let local = self.get_local(&var);
        if var.starts_with("$") || !local.is_some() {
            // set global
            bvm!(self).code.push(VmOpcode::OP_GET_GLOBAL);
            bvm!(self).cpushs(if var.starts_with("$") { &var[1..] }
                           else { var.as_str() });
        } else {
            let local = local.unwrap();
            let slot = local.0;
            let relascope = local.1;
            if relascope == 0 {
                bvm!(self).code.push(VmOpcode::OP_GET_LOCAL);
                bvm!(self).cpush16(slot);
            } else {
                bvm!(self).code.push(VmOpcode::OP_GET_LOCAL_UP);
                bvm!(self).cpush16(slot);
                bvm!(self).cpush16(relascope);
            }
        }
    }

    // labels
    pub fn reserve_label(&mut self) -> usize {
        let pos = bvm!(self).code.len();
        bvm!(self).cpush32(0xdeadbeef);
        pos
    }
    pub fn reserve_label16(&mut self) -> usize {
        let pos = bvm!(self).code.len();
        bvm!(self).cpush16(0);
        pos
    }
    pub fn fill_label16(&mut self, pos: usize, label: u16) {
        bvm!(self).cfill_label16(pos, label);
    }

    // scopes
    pub fn scope(&mut self) {
        self.scopes.push(Scope::new());
    }
    pub fn unscope(&mut self) -> u16 {
        let size = self.scopes.pop().unwrap().vars.len();
        size as u16
    }

    // loops
    pub fn loop_start(&mut self) {
        self.loop_stmts.push(LoopStatement{
            fill_continue: Vec::new(),
            fill_break: Vec::new(),
        });
    }
    pub fn loop_continue(&mut self) {
        let label = self.reserve_label();
        let ls = self.loop_stmts.last_mut().unwrap();
        ls.fill_continue.push(label);
    }
    pub fn loop_break(&mut self) {
        let label = self.reserve_label();
        let ls = self.loop_stmts.last_mut().unwrap();
        ls.fill_break.push(label);
    }
    pub fn loop_end(&mut self, next_it_pos : usize, end_pos : usize) {
        let ls = self.loop_stmts.pop().unwrap();
        for label in ls.fill_continue {
            self.fill_label16(label, (next_it_pos - label) as u16);
        }
        for label in ls.fill_break {
            self.fill_label16(label, (end_pos - label) as u16);
        }
    }

    // source map
    pub fn lookup_smap(&self, bc_idx: usize) -> Option<&SourceMap> {
        // TODO: fix this and maybe use binary search?
        let mut last_found : Option<&SourceMap> = None;
        for smap in self.smap.iter() {
            if (smap.bytecode.0..=smap.bytecode.1).contains(&bc_idx) {
                // this is so that the lookup gets more "specific"
                last_found = Some(smap);
            }
        }
        if last_found.is_some() { last_found }
        else { None }
    }
}
