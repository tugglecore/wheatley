# Wheatley
 > Storing static assets like wheat in a silo!

## Demo
```
// Embedding assets into Enum Variants
#[derive(wheatley::Wheatley)]
enum Cars;

fn random_car_fact(model: String) {
    let car = Cars::from(model).unwrap();

    match car {
        Cars::Civic => println!("The automaker of Civic is Honda"),
        Cars::Forte => println!("The body style of the Forte is sedan),
    }

    let car_history = car.contents();

    println!("Here is a breif history of {model}: {car_history}");
}

fn main () {
    // Embedding assets into Hash table
    let silo = wheatley::embed_assets! { location: "book_summaries" };

    let cat_in_the_hat_summary = silo.get("cat_in_the_hat").unwrap();

    println!("The Cat In The Hat is about {cat_in_the_hat_summary}");

    random_car_fact(String::from("civic"));
}
```

## Goal
Embed static, structured assets at compile time into a Hashmap or as enum variants.

## Highlights

 - Fastest lookup perfmance via asset hashing
 - Compress assets with br, gzip, snap or zstd
 - Filter assets with globs or files
 - Compile-time check of file existence
 - Embed structred data from a TOML file
 - And many more goodies below

## Simple Setup
First:
```
cargo add wheatley
```

Second:
Create a directory within relative to the project root which holds assets for which you will like to embed into your program.

Third:
Decide to use either enum variants or hash table. (See usage section for advice on which to choose)


## Usage

**Hash table vs Enum variants**

Briefly, recommended to use the Hash table first. Use enum variants only if the Hash table does not serve your needs.

Wheatley has two different API you can choose from: deriving enum variants or creating a hash table. It is the author recommendation to use the hash table interface since it yeilds the most performance which matters when you have a large number assets. On the other hand, use the enum variants when you would like to program logic based on different type of assets since it is more idomatic to condition logic with enum variants. 

Here is a personal antidote that will help illustrate use case. For my web server which serve static web assets, I use the hash table because I need fast lookup of assets via a path. Whereas with my internal developer tool, it has a small set of files describing common developer mistakes and I use the enum variants since each file can provide a friendly message describing the mistake made while also allowing the program to contain conditional logic for some mistakes to be fixable by the program itself.


### Options

There are several different options and feature flags to configure wheatley. Some options applies to both the enum variant approach and the Hash table while others are only applicable to one approach. 

#### Common options

#### Hash table options
**prepend_slash**
```
type: bool
default: false
```
Specifies whether each ass


#### Enum variant options




## Prior Art
 - [include_dir](https://crates.io/crates/include_dir)
 - [rust-embed](https://crates.io/crates/rust-embed)
 - [includedir](https://crates.io/crates/includedir)
 - [iftree](https://crates.io/crates/iftree)
 - [resource](https://crates.io/crates/resource)
 - [staticfilemap](https://crates.io/crates/staticfilemap)
 - [reinda](https://crates.io/crates/reinda)
 - [static-toml](https://crates.io/crates/static-toml)

## Todo list
 - use include_bytes (https://github.com/Michael-F-Bryan/include_dir/issues/20)
 - Include License in published crate (https://github.com/Michael-F-Bryan/include_dir/issues/96)
 - search for files with a glob at runtime
