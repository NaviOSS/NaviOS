// it is simple, it just takes a module and takes all of its functions!

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Item, ItemMod};

#[proc_macro_attribute]
pub fn test_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = parse_macro_input!(item as ItemMod);

    let mut content = module.content.take().unwrap();

    let func_names: Vec<_> = content
        .1
        .iter()
        .filter_map(|x| {
            if let Item::Fn(func) = x {
                Some(func.sig.ident.clone())
            } else {
                None
            }
        })
        .collect();
    let len = func_names.len();
    let test_main: Item = parse_quote! {
        pub fn test_main() {
            println!("running {} tests...", #len);
            #(
                println!("running {} test...", stringify!(#func_names));
                #func_names();
                println!("[ok]");
            )*
        }
    };

    content.1.push(test_main);

    module.content = Some(content);
    TokenStream::from(quote! {#module})
}
