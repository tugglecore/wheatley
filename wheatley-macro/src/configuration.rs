use super::GlobGroup;
use std::collections::HashMap;
use std::path::Component;
use std::path::PathBuf;
use syn::{
    parse2, 
    ExprArray, 
    LitStr,
    Expr::Lit,
    Lit::Str
};
use proc_macro2::TokenStream;

#[derive(Debug, Default)]
pub struct Config {
    pub location: PathBuf,
    pub prefix: String,
    pub use_backslash_in_keys: bool,
    pub ignore_globs: Vec<String>,
    pub asset_manifest: Vec<String>,
}

pub fn build_config(ast: proc_macro2::TokenStream) -> Config {
    let token_index = make_token_index(ast.clone());

    let use_backslash_in_keys = token_index
        .get("use_backslash_in_keys")
        .map(|token| {
            token
                .to_string()
                .parse::<bool>()
                .expect("Config value for use_backslash_in_keys is a bool type")
        })
        .unwrap_or(false);

    let prefix = token_index
        .get("prefix")
        .map(|token| {
            token
                .to_string()
                .parse::<String>()
                .expect("Config value for prepend_slash is a string type")
        })
        .unwrap_or_default();


    let ignore_globs = token_index.get("ignore_globs")
        .map(|token_tree| proc_macro2::TokenStream::from(token_tree.clone()));

    let survey_as_glob_list = |token_stream: TokenStream| {
        parse2::<ExprArray>(token_stream)
            .ok()
            .map(|expr_array| {
                expr_array
                    .elems
                    .into_iter()
                    .map(|e| {
                        if let syn::Expr::Lit(literal) = e {
                            if let Str(glob) = literal.lit {
                                return glob.value()
                            }
                        }
                        panic!("asdf")
                })
                .collect::<Vec<String>>()
            })
    };

    let survey_as_glob_file = |token_stream: TokenStream| {
        parse2::<LitStr>(token_stream)
            .ok()
            .map(|lit_str| vec![lit_str.value()])
    };

    let ignore_globs = ignore_globs
        .and_then(|token_stream| {
            let transformations = [
                survey_as_glob_list(token_stream.clone()),
                survey_as_glob_file(token_stream.clone())
            ]
            .into_iter()
            .filter(|t| t.is_some())
            .collect::<Vec<_>>();

            if transformations.len() != 1 { panic!("sfs") }

            transformations
                .first()
                .unwrap()
                .clone()
        })
        .unwrap_or_default();

    // let ignore_globs = if let Some(token) = token_index.get("ignore_globs") {
    //     let token_stream = proc_macro2::TokenStream::from(token.clone());
    //     if let Ok(list) = parse2::<ExprArray>(token_stream.clone()) {
    //         list
    //             .elems
    //             .iter()
    //             .map(|exp| {
    //                 match exp {
    //                     syn::Expr::Lit(literal)
    //                         if matches!(literal.lit, syn::Lit::Str(_)) => {
    //                             String::from("fsd")
    //                         }
    //                     _ => { panic!("f") }
    //                 }
    //             })
    //             .collect::<Vec<String>>()
    //     } else if let Ok(file_path) = parse2::<LitStr>(token_stream.clone()) {
    //         vec![]
    //     } else {
    //         panic!("Failed to interpret ignore_globs value {token:#?}")
    //     }
    // } else {
    //     vec![]
    // };

    let location = token_index
        .get("location")
        .map(|token| parse2::<LitStr>(token.clone().into()).unwrap().value())
        .map(PathBuf::from)
        .expect("Missing assets directory")
        .components()
        .map(|component| match component {
            Component::Normal(segment) if segment.to_str().is_some_and(|s| s.starts_with("$")) => {
                let segment = &segment.to_str().unwrap()[1..];
                std::env::var(segment).expect(
                    "
                            failed attempt to interpret {segment} as enviroment variable
                        ",
                )
            }
            _ => component.as_os_str().to_string_lossy().into_owned(),
        })
        .collect::<PathBuf>();

    Config {
        location,
        ignore_globs,
        asset_manifest: vec![],
        prefix,
        use_backslash_in_keys,
    }
}

fn make_token_index(ast: proc_macro2::TokenStream) -> HashMap<String, proc_macro2::TokenTree> {
    let mut attribute_index = HashMap::new();

    ast.into_iter()
        .filter(|token| {
            matches!(
                token,
                proc_macro2::TokenTree::Group(_)
                    | proc_macro2::TokenTree::Literal(_)
                    | proc_macro2::TokenTree::Ident(_)
            )
        })
        .collect::<Vec<proc_macro2::TokenTree>>()
        .chunks(2)
        .for_each(|pair| {
            if let [k, v] = pair {
                attribute_index.insert(k.to_string(), v.clone());
            } else {
                panic!("failed to interpret input {pair:#?}")
            }
        });

    attribute_index
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use test_case::test_case;

    #[test_case(
        "$CARGO_MANIFEST_DIR/assets",
        &(std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/assets");
        "env based location"
    )]
    #[test_case("./assets", "./assets"; "simple location")]
    fn read_location(given_location: &str, expected_location: &str) {
        let expected_location = PathBuf::from(expected_location);
        let ast = quote! {
            location: #given_location
        };

        let Config {
            location: actual_location,
            ..
        } = build_config(ast);

        assert_eq!(actual_location, expected_location);
    }
}
