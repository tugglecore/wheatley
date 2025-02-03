mod compression;
mod configuration;
mod mphf;

use self::compression::compress_assets;
use configuration::Config;
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

    let assets = gather(&config)
        .drain()
        .collect::<Vec<(String, Vec<u8>)>>();

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

fn gather(config: &Config) -> HashMap<String, Vec<u8>> {
    let Config {
        location,
        prefix,
        use_backslash_in_keys,
        ..
    } = config;

    let mut asset_register = HashMap::new();
    let mut dirs = std::collections::VecDeque::from([location.to_path_buf()]);
    dbg!(&dirs);

    while let Some(dir) = dirs.pop_front() {

        dbg!(&dir);
        for entry in std::fs::read_dir(dir).unwrap() {
            dbg!(&entry);
            let entry = entry.unwrap();
            println!("Potential path: {:#?}", entry.path());
            println!("Strip prefix: {:#?}", entry.path().strip_prefix(location));

            let file_type = entry.file_type().unwrap();

            let mut path = entry.path();
            if file_type.is_file() {
                let asset = std::fs::read(&path).unwrap();

                let separator = if cfg!(windows) && *use_backslash_in_keys {
                    r"\"
                } else {
                    "/"
                };

                let asset_key = path
                    .strip_prefix(location)
                    .unwrap()
                    .to_path_buf()
                    .components()
                    .filter(|c| matches!(c, Component::Normal(_)))
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(separator);

                let asset_key = prefix.clone() + &asset_key;
                dbg!(&asset_key);
                dbg!(&prefix);

                asset_register.insert(asset_key, asset);
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
    mut assets: Vec<(String, Vec<u8>)>,
) -> (mphf::bbhash::Mphf<String>, Vec<(String, Vec<u8>)>) {
    let file_paths = assets
        .iter()
        .map(|(k, _)| k.clone())
        .collect::<Vec<String>>();

    let bbhas = mphf::bbhash::Mphf::new(1.7, &file_paths);

    for current_asset_position in 0..assets.len() {
        loop {
            let (file_path, _) = &assets[current_asset_position];
            let hash_position = bbhas.hash(
           file_path
            ) as usize;

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
    hash_table: Vec<(String, Vec<u8>)>,
) -> TokenStream {
    let entries = hash_table.iter().map(|(key, value)| {
        let path = key.as_bytes();
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

    Config {
        location: location.into(),
        ..Default::default()
    }
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
    use fake::faker::filesystem::en::FileName;
    use fake::Fake;
    use std::io::{LineWriter, Write};
    use tempfile::{NamedTempFile, Builder};
    use test_case::test_case;
    use tempfile::TempDir;

    fn create_assets(assets: &[(&str, &str)]) -> Vec<(String, Vec<u8>)> {
        assets
            .iter()
            .map(|(p, c)| (p.clone().to_owned(), Vec::from(c.as_bytes())))
            .collect::<Vec<(String, Vec<u8>)>>()
    }

    fn shuffle(mut buoy: Vec<(String, Vec<u8>)>) -> Vec<(String, Vec<u8>)> {
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

    fn build_fake_keys(sub_dir: &TempDir, fake_file: &NamedTempFile, separator: &str, prefix: &str) -> Vec<String> {
        let asset_path = [
            // prefix.to_owned(),
            sub_dir
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            fake_file
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        ]
        .join(separator);

        let mut fake_key = String::from(prefix);
        fake_key.push_str(&asset_path);

        vec![fake_key]
    }

    #[test_case(false, "/", "/prefix/"; "forward slash with prefix")]
    #[test_case(false, "/", ""; "forward slash without prefix")]
    #[test_case(true, r"\", "/prefix/"; "backslash with prefix")]
    #[test_case(true, r"\", ""; "backslash withouth prefix")]
    fn test_path_separators(use_backslash_separator: bool, separator: &str, prefix: &str) {
        let tmp_dir = TempDir::with_prefix("wheatley_location").unwrap();
        let sub_dir = Builder::new()
            .prefix("wheatley_sub_dir")
            .tempdir_in(tmp_dir.path())
            .unwrap();
        let fake_file = Builder::new()
            .prefix("foobar")
            .tempfile_in(sub_dir.path())
            .unwrap();

        let expected_keys = build_fake_keys(
            &sub_dir, 
            &fake_file,
            separator,
            prefix
        );
        
        let config = Config {
            location: tmp_dir.path().to_owned(),
            use_backslash_in_keys: use_backslash_separator,
            prefix: prefix.to_owned(),
            ..Default::default()
        };

        let actual_keys = gather(&config)
            .into_keys()
            .collect::<Vec<_>>();

        assert_eq!(
            actual_keys,
            expected_keys
        )
    }
}
