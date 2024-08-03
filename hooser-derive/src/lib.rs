use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use std::fs;
use toml::Table;


#[proc_macro_derive(Hooser)]
pub fn hooser(tokens: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(tokens as DeriveInput);
    let identifer = ast.ident;

    let asset = fs::read_to_string("./lucille.toml").unwrap();
    let asset = asset.parse::<Table>().unwrap();
    let bio = asset["bio"].as_str();

    let gen = quote! {
        impl #identifer {
            fn bio (&self) -> &'static str {
                // "Poet"
                #bio
            }
        }
    };

    gen.into()
}

