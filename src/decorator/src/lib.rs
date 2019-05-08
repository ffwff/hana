extern crate proc_macro;
#[macro_use] extern crate quote;
use proc_macro::TokenStream;
use syn;

#[proc_macro_attribute]
pub fn hana_function(_args: TokenStream, item: TokenStream) -> TokenStream {
    /*
    #[hana_function()]
    fn fopen(path: Value::String, mode: Value::String) {
        ...
    }

    should generate a function like this (semi pseudocode):
    pub extern "C" fn fopen(cvm : *mut Vm, nargs : u16) {
        if nargs != [nargs] { panic!(...) }
        fn fopen() -> Value {
            let Value::String(path) = vm.stack.pop().unwrap() ||
                    panic!("expected path to be string");
            let Value::String(mode) = vm.stack.pop().unwrap() ||
                    panic!("expected mode to be string");
            // call fopen(path, mode)
        }
        let vm = unsafe { &mut *cvm };
        let result : Value = #name(vm);
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
                let pattern = match &cap.pat {
                    syn::Pat::Ident(x) => x,
                    _ => panic!("expected identifier argument!")
                };
                let path = match &cap.ty {
                    syn::Type::Path(x) => &x.path.segments,
                    _ => panic!("expected type for {:?} to be path!", pattern)
                };
                // match and unwrap type from value variant
                // also panics if unexpected type
                let atype = path.last().unwrap().into_value().ident.to_string();
                let atypes = syn::LitStr::new(atype.as_str(),
                                    quote::__rt::Span::call_site());
                let argname = syn::LitStr::new(pattern.ident.to_string().as_str(),
                                    quote::__rt::Span::call_site());
                let match_arm = match atype.as_str() {
                    "Int" | "Float" | "NativeFn" | "Fn" | "Str" | "Dict" | "Array"
                        => quote!(#path(x)),
                    _ => quote!(#path)
                };
                args_setup.push(match atype.as_str() {
                    "Any" => quote!(let #pattern = {
                        let val = vm.stack.top().unwrap();
                        vm.stack.pop();
                        val
                    };),
                    _ => {
                        quote!(
                            let #pattern = {
                                let val = vm.stack.top().unwrap();
                                vm.stack.pop();
                                match val {
                                    #match_arm => x,
                                    _ => panic!("expected argument {} to be type {}",
                                        #argname,
                                        #atypes)
                                }
                            };
                        )
                    },
                });
            },
            _ => unimplemented!()
        }
    }

    let arglen = syn::LitInt::new(input.decl.inputs.len() as u64,
                        syn::IntSuffix::None,
                        quote::__rt::Span::call_site());

    quote!(
        pub extern "C" fn #name(cvm : *mut Vm, nargs : u16) {
            if nargs != #arglen {
                panic!("unmatched arguments length, expected {}!", #arglen);
            }
            fn #name(vm: &mut Vm) -> Value {
                #(#args_setup)*
                #body
            }
            let vm = unsafe { &mut *cvm };
            let result : Value = #name(vm);
            vm.stack.push(result.wrap());
        }
    ).into()
}