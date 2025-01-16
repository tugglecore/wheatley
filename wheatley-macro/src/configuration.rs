use super::GlobGroup;
use std::collections::HashMap;
use std::path::Component;
use std::path::PathBuf;
use syn::{parse2, ExprArray, LitStr};

#[derive(Debug, Default)]
pub struct Con {
    pub location: PathBuf,
    pub prepend_slash: bool,
    pub use_backslash_in_keys: bool,
    pub ignore_globs: Vec<String>,
    pub asset_manifest: Vec<String>,
}

pub fn build_config(ast: proc_macro2::TokenStream) -> Con {
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

    let prepend_slash = token_index
        .get("prepend_slash")
        .map(|token| {
            token
                .to_string()
                .parse::<bool>()
                .expect("Config value for prepend_slash is a bool type")
        })
        .unwrap_or(false);

    // let mut include_globs = HashSet::new();

    let include_globs = token_index
        .get("include_globs")
        .map(|token_tree| {
            let proc_macro2::TokenTree::Group(group) = token_tree else {
                panic!("fixme")
            };

            group
                .stream()
                .into_iter()
                .map(|item| parse2::<syn::LitStr>(item.into()).unwrap().value())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    // let include_manifest = token_index
    //     .get("include_manifest")
    //     .map(|token| parse::<syn::LitStr>(token.clone().into()).unwrap().value())
    //     .or(Some("include_manifest.txt".into()))
    //     .map(|path| {
    //         let path = Path::from(path);
    //         let include_patterns = fs::read_to_string(path)
    //             .unwrap()
    //             .lines()
    //             .
    //     })
    //     .map(PathBuf::from)
    //     .expect("Missing assets directory");

    // let mut include_builder = GlobSetBuilder::new();
    //
    // for glob in include_globs {
    //     let glob = Glob::new(&glob).unwrap();
    //     include_builder.add(glob);
    // }
    // let include_globs = include_builder.build().unwrap();

    let include_globs = GlobGroup::new(&include_globs);
    let exclude_globs = token_index
        .get("exclude_globs")
        .map(|token_tree| {
            let proc_macro2::TokenTree::Group(group) = token_tree else {
                panic!("fixme")
            };

            group
                .stream()
                .into_iter()
                .map(|item| parse2::<syn::LitStr>(item.into()).unwrap().value())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    // let mut builder = GlobSetBuilder::new();
    //
    // for glob in exclude_globs {
    //     let glob = Glob::new(&glob).unwrap();
    //     builder.add(glob);
    // }
    // let exclude_globs = builder.build().unwrap();
    let exclude_globs = GlobGroup::new(&exclude_globs);

    // let ignore_globs = token_index
    //     .get("ignore_globs")
    //     .map(|token| {
    //         parse2::<ExprArray>(token.clone().into())
    //             .unwrap()
    //             .elems
    //             .iter()
    //             .collect::<

    let ignore_globs = if let Some(token) = token_index.get("ignore_globs") {
        let token_stream = proc_macro2::TokenStream::from(token.clone());
        if let Ok(list) = parse2::<ExprArray>(token_stream.clone()) {
            list
                .elems
                .iter()
                .map(|exp| {
                    match exp {
                        syn::Expr::Lit(literal)
                            if matches!(literal.lit, syn::Lit::Str(_)) => {
                                String::from("fsd")
                            }
                        _ => { panic!("f") }
                    }
                })
                .collect::<Vec<String>>()
        } else if let Ok(file_path) = parse2::<LitStr>(token_stream.clone()) {
            vec![]
        } else {
            panic!("Failed to interpret ignore_globs value {token:#?}")
        }
    } else {
        vec![]
    };

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

    Con {
        location,
        ignore_globs,
        asset_manifest: vec![],
        prepend_slash,
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

        let Con {
            location: actual_location,
            ..
        } = build_config(ast);

        assert_eq!(actual_location, expected_location);
    }
}
