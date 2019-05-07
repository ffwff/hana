extern crate proc_macro;
#[macro_use] extern crate quote;
use proc_macro::TokenStream;
use syn;

#[proc_macro_attribute]
pub fn hana_function(args: TokenStream, item: TokenStream) -> TokenStream {
    /*
    #[hana_function()]
    fn fopen(path: Value::String, mode: Value::String) {
        ...
    }

    should generate a function like this:
    fn fopen(vm : &mut Vm, nargs : u16) {
        if nargs != [nargs] { panic!(...) }
        let path = vm.stack.pop().unwrap().string();
        let mode = vm.stack.pop().unwrap().string();
        fn fopen(path, mode) -> Value {
            // call fopen(path, mode)
        }
        let result = fopen(path, mode);
        vm.stack.push(result.wrap());
    }
    */

    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.ident;
    let body = &input.block;

    let mut args_setup = Vec::new();
    for arg in input.decl.inputs.iter() {
        match *arg {
            syn::FnArg::Captured(ref cap) => {
                let pat = &cap.pat;
                let ty = &cap.ty;
                args_setup.push(match ty {
                    syn::Type::Path()
                    quote!(let #pat = vm.stack.pop().unwrap();)
                });
            }
        }
    }
    let arglen = syn::LitInt((&input.decl.inputs).len());

    quote!(
        fn #name(vm : &mut Vm, nargs : u16) {
            if nargs != #arglen { panic!("unmatched arguments length!"); }
            fn #name() {
                #body
            }
        }
    ).into()
}