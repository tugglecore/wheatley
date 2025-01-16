mod compression;
mod configuration;
mod mphf;

use self::compression::compress_assets;
use configuration::Con;
use globset::{Glob, GlobSet, GlobSetBuilder};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::path::PathBuf;
use std::path::{Component, Path};
use std::{fmt, fs};
use syn::{parse_macro_input, DeriveInput};
use toml::Table;

#[derive(Debug)]
struct Config {
    location: String,
}

#[derive(Default)]
struct GlobGroup {
    patterns: Vec<String>,
    glob_set: GlobSet,
}

impl GlobGroup {
    fn new(globs: &[String]) -> Self {
        let mut glob_set = GlobSetBuilder::new();
        let mut patterns = Vec::new();
        for glob in globs {
            patterns.push(glob.to_string());

            glob_set.add(Glob::new(glob).unwrap());
        }

        let glob_set = glob_set.build().unwrap();

        Self { patterns, glob_set }
    }

    fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.glob_set.is_match(path)
    }

    fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

impl PartialEq for GlobGroup {
    fn eq(&self, other: &Self) -> bool {
        self.patterns == other.patterns
    }
}

impl fmt::Debug for GlobGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlobGroup")
            .field("patterns", &self.patterns)
            .finish()
    }
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
    let config = configuration::build_config(token_stream.into());

    println!("I AM HERE BRO!!!");

    let assets = gather(&config).drain().collect::<Vec<(PathBuf, Vec<u8>)>>();

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

fn gather(config: &Con) -> HashMap<PathBuf, Vec<u8>> {
    let Con {
        location,
        prepend_slash,
        use_backslash_in_keys,
        ..
    } = config;

    let mut asset_register = HashMap::new();
    let mut dirs = std::collections::VecDeque::from([location.to_path_buf()]);

    // let we_have_include_globs = !include_globs.is_empty();
    // println!("Do we have include globs {we_have_include_globs:#?}");
    while let Some(dir) = dirs.pop_front() {
        // println!(
        //     "Does glob match path {:#?}: {:#?}",
        //     dir,
        //     include_globs.is_match(&dir)
        // );

        // if !include_globs.is_empty() && !include_globs.is_match(&dir) {
        //     continue;
        // }

        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            println!("Potential path: {:#?}", entry.path());
            println!("Strip prefix: {:#?}", entry.path().strip_prefix(location));

            let asset_relative_path = entry.path();

            let asset_relative_path = asset_relative_path.strip_prefix(location).unwrap();

            // if exclude_globs.is_match(asset_relative_path) || (we_have_include_globs && !include_globs.is_match(asset_relative_path)) {
            //     println!("We continued for {asset_relative_path:#?}");
            //         continue
            // }

            let file_type = entry.file_type().unwrap();

            let mut path = if cfg!(windows) && *use_backslash_in_keys {
                let path = entry.path();

                if path.to_string_lossy().contains("/") {
                    split_path_with_separator(path, r"\")
                } else {
                    path
                }
            } else {
                split_path_with_separator(entry.path(), "/")
            };

            if file_type.is_file() {
                let asset = std::fs::read(entry.path()).unwrap();

                if *prepend_slash {
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

fn split_path_with_separator(path: PathBuf, separator: &str) -> PathBuf {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(separator)
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
    use assertables::*;
    use fake::faker::filesystem::en::FileName;
    use fake::Fake;
    use std::io::{LineWriter, Write};
    use tempfile::NamedTempFile;
    use test_case::test_case;

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

    fn write_lines_to_temp_file(lines: &[&str]) -> String {
        let mut temp_file = NamedTempFile::with_prefix("something").unwrap();

        for line in lines {
            writeln!(temp_file, "{}", line).unwrap();
        }

        temp_file.path().to_string_lossy().into()
    }

    fn to_vec_of_strings(slices: &[&str]) -> Vec<String> {
        slices.iter().map(|i| (*i).to_owned()).collect()
    }

    fn write_fake_directory(entries: &[&str]) -> String {
        let location: String = FileName().fake();
        let tmp_dir = tempfile::Builder::new()
            .prefix(&location)
            .tempdir_in("./")
            .unwrap();

        for entry in entries {
            tempfile::Builder::new()
                .prefix(entry)
                .tempfile_in(&tmp_dir)
                .unwrap();
        }

        tmp_dir.path().to_str().unwrap().into()
    }

    // #[test_case([], [], ["a", "b", "c"]; "wo")]
    // fn somthing (
    //     wheatley_ignore: Vec<&str>,
    //     wheatley_manifest: Vec<&str>,
    //     expected_files: Vec<&str>
    // ) {
    //     let config = configuration::Con {
    //         location: "filter_fixtures",
    //         wheatley_ignore,
    //         wheatley_manifest,
    //         ..Default::default()
    //     };
    //     let collected_files: Vec<String> = gather(config)
    //         .keys()
    //         .map(|k| k.to_string_lossy().into_owned())
    //         .collect::<_>();
    //
    //     assert_eq!(collected_files, expected_files);
    // }
}
