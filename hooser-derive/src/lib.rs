use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Hooser)]
pub fn hooser(tokens: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(tokens as DeriveInput);
    let identifer = ast.ident;

    let gen = quote! {
        impl #identifer {
            fn bio (&self) -> &'static str {
                "Poet"
            }
        }
    };

    gen.into()
}

