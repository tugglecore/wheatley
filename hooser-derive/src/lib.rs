use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use syn::{parse_macro_input, DeriveInput};
use toml::Table;

#[proc_macro_derive(Hooser)]
pub fn hooser(tokens: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(tokens as DeriveInput);
    let target = ast.ident.clone();

    let mut static_assets_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut identifier = ast.ident.clone().to_string();
    identifier.make_ascii_lowercase();
    static_assets_path.push(identifier);

    // Map file names to file content

    let mut asset_register: HashMap<String, toml::Table> = HashMap::new();

    for entry in fs::read_dir(&static_assets_path).unwrap() {
        let entry = entry.unwrap();
        let unstructured_data = fs::read_to_string(entry.path()).unwrap();
        let asset = unstructured_data.parse::<Table>().unwrap();

        asset_register.insert(
            entry
                .path()
                .file_name()
                .and_then(|file_name| Path::new(file_name).file_stem())
                .and_then(std::ffi::OsStr::to_str)
                .map(|file_stem| file_stem.to_snake_case())
                .unwrap(),
            asset,
        );
    }

    // Map variant names to variant data

    let mut variant_register = HashMap::new();

    let syn::Data::Enum(target_enum) = ast.data else {
        panic!("Can only derive an Enum")
    };

    for variant in target_enum.variants.iter() {
        variant_register.insert(variant.ident.to_string().to_snake_case(), variant.clone());
    }

    // TODO: Confirm there is a 1 to 1 mapping
    // between files and variants and there is
    // no non-matching item in either set

    // TODO: Confirm all files have the same schema

    let properties = asset_register
        .values()
        .next()
        .map(|fc| {
            fc.keys()
                .map(std::clone::Clone::clone)
                .collect::<Vec<String>>()
        })
        .unwrap();

    let mut fragments = vec![];

    for property in properties {
        let mut variant_matchers = vec![];

        for (variant_name, variant) in variant_register.iter() {
            let file_contents = asset_register.get(variant_name).unwrap();
            let asset = file_contents.get(&property).unwrap().as_str().unwrap();
            let v = &variant.ident;
            let i = &target;
            let variant_matcher = quote! {
                #i::#v => #asset
            };

            variant_matchers.push(variant_matcher);
        }

        let property = quote::format_ident!("{}", property);

        let property_method = quote! {
            fn #property (&self) -> &'static str {
                match self {
                    #(#variant_matchers),*
                }
            }
        };

        fragments.push(property_method);
    }

    let output = quote! {
        impl #target {
            #(#fragments)*
        }
    };

    output.into()
}
