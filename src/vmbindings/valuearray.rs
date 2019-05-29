use super::carray::CArray;
use super::cnativeval::NativeValue;
use super::gc::Gc;
use super::vm::Vm;
use super::value::Value;

#[derive(PartialEq, Clone)]
pub struct ValueArray {
    data: Gc<CArray<NativeValue>>,
}

impl ValueArray {

    pub fn malloc(vm: &Vm) -> ValueArray {
        ValueArray {
            data: vm.malloc(CArray::new())
        }
    }

    pub fn from_carray(data: Gc<CArray<NativeValue>>) -> ValueArray {
        ValueArray {
            data,
        }
    }

    pub fn data(&self) -> &Gc<CArray<NativeValue>> {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.as_ref().len()
    }

    // stack manip
    pub fn push(&mut self, val: Value) {
        self.data.as_mut().push(val.wrap())
    }

    pub fn pop(&mut self) {
        self.data.as_mut().pop();
    }

    pub fn top(&self) -> Option<Value> {
        Some(self.data.as_ref().top().unwrap())
    }

}

impl std::ops::Index<usize> for ValueArray {
    type Output = Value;
    fn index(&self, i: usize) -> Self::Output {
        self.data.as_ref()[i].unwrap()
    }
}