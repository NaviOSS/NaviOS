// it is simple, it just takes a module and takes all of its functions!

use core::panic;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, Item, ItemMod};

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

#[proc_macro_derive(EncodeKey)]
pub fn derive_encode_key(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let name = item.ident;

    let data = match item.data {
        syn::Data::Enum(data) => data,
        _ => panic!("expected an enum"),
    };

    let arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            quote! { Self::#ident => KeyCode::#ident, }
        })
        .collect();

    TokenStream::from(quote! {
        impl EncodeKey for #name {
            fn encode(self) -> KeyCode {
                match self {
                    #(#arms)*
                }
            }
        }
    })
}
