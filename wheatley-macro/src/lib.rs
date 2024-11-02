mod compression;
mod mphf;

use self::compression::compress_assets;
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::path::{Component, Path};
use syn::{parse_macro_input, DeriveInput};
use toml::Table;

#[derive(Debug)]
struct Config {
    location: String,
}

#[derive(Debug)]
struct Con {
    location: PathBuf,
    prepend_slash: bool,
    compression_unit: Option<String>,
    use_windows_path_separator: bool,
}

#[proc_macro_derive(Hooser, attributes(wheatley))]
pub fn hooser(tokens: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(tokens as DeriveInput);

    let output = match ast.data {
        syn::Data::Enum(_) => write_enum_impl(ast),
        _ => unimplemented!("Only Enum is supported."),
    };

    output.into()
}

#[proc_macro]
pub fn embed_assets(token_stream: TokenStream) -> TokenStream {
    let config = build_config(token_stream.clone());

    let mut asset_register = gather(
        &config.location,
        config.prepend_slash,
        config.use_windows_path_separator,
    );

    let assets = asset_register.drain().collect::<Vec<(PathBuf, Vec<u8>)>>();

    let (hasher, mut hash_table) = build_hash_table(assets);

    if cfg!(any(
        feature = "gzip",
        feature = "br",
        feature = "snap",
        feature = "zstd",
    )) {
        compress_assets(&mut hash_table);
    }

    tokenize_hash_components(hasher, hash_table)
}

fn build_config(ast: TokenStream) -> Con {
    let token_index = make_token_index(ast.clone());

    let use_windows_path_separator = token_index
        .get("use_windows_path_separator")
        .map(|token| {
            token
                .to_string()
                .parse::<bool>()
                .expect("Config value for use_windows_path_separator is a bool type")
        })
        .unwrap_or(false);

    // got some && should do it: Done
    // got none && should do it: Done
    // got none && don't do it: Done
    // got some && don't do it: Done
    let compression_unit = token_index
        .get("compression_unit")
        .inspect(|_| {})
        .map(|token| token.to_string().trim_matches('"').to_string())
        .inspect(|unit| {
            if !matches!(unit.as_str(), "file" | "directory") {
                panic!("Compression unit can either be file or directory");
            }
        })
        .or_else(|| Some("file".to_string()));

    let prepend_slash = token_index
        .get("prepend_slash")
        .map(|token| {
            token
                .to_string()
                .parse::<bool>()
                .expect("Config value for prepend_slash is a bool type")
        })
        .unwrap_or(false);

    let location = token_index
        .get("location")
        .map(|token| token.to_string().trim_matches('"').to_string())
        .expect("Missing assets directory")
        .split(std::path::is_separator)
        .map(|component| {
            if component.starts_with("$") {
                std::env::var(component).unwrap()
            } else {
                component.to_string()
            }
        })
        .collect::<std::path::PathBuf>();

    Con {
        location,
        prepend_slash,
        compression_unit,
        use_windows_path_separator,
    }
}

fn gather(
    location: &Path,
    prepend_slash: bool,
    use_windows_path_separator: bool,
) -> HashMap<PathBuf, Vec<u8>> {
    let mut asset_register = HashMap::new();
    let mut dirs = std::collections::VecDeque::from([location.to_path_buf()]);

    while let Some(dir) = dirs.pop_front() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();

            if file_type.is_file() {
                let asset = std::fs::read(entry.path()).unwrap();

                let mut path = if use_windows_path_separator && cfg!(windows) {
                    entry.path()
                } else {
                    to_unix_path(entry.path())
                };

                if prepend_slash {
                    path = Path::new("/").join(path);
                }

                asset_register.insert(path, asset);
            } else if file_type.is_symlink() {
                panic!(
                    "Encountered Symlink at: {}. Symlinks are not supported.",
                    entry.path().to_str().unwrap()
                );
            } else {
                dirs.push_back(entry.path());
            }
        }
    }

    asset_register
}

fn build_hash_table(
    mut assets: Vec<(PathBuf, Vec<u8>)>,
) -> (mphf::bbhash::Mphf<String>, Vec<(PathBuf, Vec<u8>)>) {
    let file_paths = assets
        .iter()
        .map(|(p, _)| p.to_string_lossy().into_owned())
        .collect::<Vec<String>>();

    let bbhas = mphf::bbhash::Mphf::new(1.7, &file_paths);

    for current_asset_position in 0..assets.len() {
        loop {
            let (file_path, _) = &assets[current_asset_position];
            let hash_position = bbhas.hash(&file_path.to_string_lossy().into_owned()) as usize;

            if current_asset_position == hash_position {
                break;
            }

            assets.swap(hash_position, current_asset_position);
        }
    }

    (bbhas, assets)
}

fn tokenize_hash_components(
    hasher: mphf::bbhash::Mphf<String>,
    hash_table: Vec<(PathBuf, Vec<u8>)>,
) -> TokenStream {
    let entries = hash_table.iter().map(|(key, value)| {
        let path = key.to_str().unwrap().as_bytes();
        let contents = value.as_slice();
        quote! {
            wheatley::Entry::File(
                wheatley::File::new(
                    &[ #(#path),* ],
                    &[ #(#contents),* ]
                )
            )
        }
    });

    let bit_vectors = hasher
        .bitvecs
        .iter()
        .map(|(bit_vector, bits)| {
            let bits = bits.into_iter();

            quote! {

                (
                    wheatley::BitVector::from_embedded_state(#bit_vector),
                    &[ #(#bits),* ]
                )
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        wheatley::Wheatley::new(
            {
                static ENTRIES: &'static [wheatley::Entry] = &[ #(#entries),* ];
                ENTRIES
            },
            {
                static BIT_VECTORS: &'static [(wheatley::BitVector, &[u64])] = &[ #(#bit_vectors),* ];
                BIT_VECTORS
            }
        )
    }
    .into()
}

fn to_unix_path(windows_path: PathBuf) -> PathBuf {
    windows_path
        .components()
        .filter_map(|c| match c {
            Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
        .into()
}

fn make_token_index(ast: TokenStream) -> HashMap<String, proc_macro::TokenTree> {
    let mut attribute_index = HashMap::new();

    ast.into_iter()
        .filter(|token| {
            matches!(
                token,
                proc_macro::TokenTree::Group(_)
                    | proc_macro::TokenTree::Literal(_)
                    | proc_macro::TokenTree::Ident(_)
            )
        })
        .collect::<Vec<proc_macro::TokenTree>>()
        .chunks(2)
        .for_each(|pair| {
            if let [k, v] = pair {
                attribute_index.insert(k.to_string(), v.clone());
            }
        });

    attribute_index
}

fn write_enum_impl(ast: DeriveInput) -> proc_macro2::TokenStream {
    let target = ast.ident.clone();
    let asset_register = gather_assets(ast.clone());
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

    output
}

fn pick_attributes(ast: DeriveInput) -> Config {
    println!("AST of DeriveInpute: {ast:#?}");
    let mut attribute_index = HashMap::new();
    ast.attrs
        .into_iter()
        .filter_map(|att| {
            let syn::Meta::List(meta_list) = att.meta else {
                return None;
            };

            if meta_list.path.segments.first().unwrap().ident != "wheatley" {
                return None;
            }

            let token_strings = meta_list
                .tokens
                .into_iter()
                .filter_map(|token| match token {
                    proc_macro2::TokenTree::Ident(ident) => Some(ident.to_string()),
                    proc_macro2::TokenTree::Literal(literal) => Some(literal.to_string()),
                    _ => None,
                })
                .collect::<Vec<String>>();
            Some(token_strings)
        })
        .flatten()
        .collect::<Vec<String>>()
        .chunks(2)
        .for_each(|pair| {
            if let [k, v] = pair {
                attribute_index.insert(k.clone(), v.trim_matches('"').to_string());
            }
        });

    println!("Tokens given to Wheatley: {attribute_index:#?}");

    let location = attribute_index.get("location").cloned().unwrap_or_else(|| {
        let mut identifier = ast.ident.clone().to_string();
        identifier.make_ascii_lowercase();

        let mut static_assets_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        static_assets_path.push(identifier);
        static_assets_path.to_string_lossy().to_string()
    });

    Config { location }
}

fn gather_assets(ast: DeriveInput) -> HashMap<String, toml::Table> {
    let assets_directory = pick_attributes(ast.clone()).location;

    // Map file names to file content

    let mut asset_register: HashMap<String, toml::Table> = HashMap::new();

    println!("{assets_directory:#?}");
    // TODO: During development read the file from disk
    // while building in production read embed the data
    for entry in fs::read_dir(assets_directory).unwrap() {
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

    asset_register
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_assets(assets: &[(&str, &str)]) -> Vec<(PathBuf, Vec<u8>)> {
        assets
            .iter()
            .map(|(p, c)| (PathBuf::from(*p), Vec::from(c.as_bytes())))
            .collect::<Vec<(PathBuf, Vec<u8>)>>()
    }

    fn shuffle(mut buoy: Vec<(PathBuf, Vec<u8>)>) -> Vec<(PathBuf, Vec<u8>)> {
        let size = buoy.len();
        let halfway_point = size / 2;
        for i in 0..halfway_point {
            println!("check math {i:#?}");
            buoy.swap(i, halfway_point - i)
        }

        buoy
    }

    #[test]
    fn confirm_entries_sorted_by_hasher() {
        let assets = create_assets(&[("foo", "bar"), ("qux", "baz"), ("zoo", "books")]);
        let (_, expected_hash_table) = build_hash_table(assets.clone());
        let remixed_assets = shuffle(expected_hash_table.clone());

        let (_, hash_table_result) = build_hash_table(remixed_assets);

        assert_eq!(expected_hash_table, hash_table_result);
    }
}
