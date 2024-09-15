# Wheatley
 > Storing and protecting safe, structured, static assets like Fort Knox!

Embed static, structured data into your program at compile time.


## Demo
```
    
```


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
 - File check at runtime or compile time
    - (https://github.com/Michael-F-Bryan/include_dir/issues/67)
    - (https://github.com/pyrossh/rust-embed/issues/242)
 - Prepend_path_with_slash (https://github.com/Michael-F-Bryan/include_dir/issues/75)
 - Rename path at runtime (https://github.com/pyrossh/rust-embed/issues/236)
 - Dynamic include/exclude list of files
    - (https://github.com/pyrossh/rust-embed/issues/221)
    - (https://github.com/Michael-F-Bryan/include_dir/issues/81)
    - (https://github.com/Michael-F-Bryan/include_dir/issues/13)
 - Offer compression mode
    - Compress entire folder or file by file basis (https://github.com/pyrossh/rust-embed/issues/248)
    - Only use zstd algorithm https://github.com/Michael-F-Bryan/include_dir/issues/14#issuecomment-1111658509
        - flate2 (gzip)
        - brotli
        - zstd
        - snap
 - use include_bytes (https://github.com/Michael-F-Bryan/include_dir/issues/20)
 - Include License in published crate (https://github.com/Michael-F-Bryan/include_dir/issues/96)
 - Option to specify directory
 - Support env variables
 - Iterators over feilds
 - search for files with a glob at runtime
