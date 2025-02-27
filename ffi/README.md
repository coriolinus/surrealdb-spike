# `checklist` FFI

Foreign Function Inteface to the `checklist` native Rust library.

## Building FFI bindings with `uniffi`

```sh
# build the .so file
cargo build --release -p ffi --features uniffi
# build the bindings
# adjust as required for other binding languages
cargo run --release \
    --features uniffi-cli \
    --bin uniffi-bindgen -- \
    generate \
    --library target/release/libchecklist_ffi.so \
    --language python \
    --out-dir ffi/bindings/python
# .so must be distributed in parallel with the bindings
ln -s "$(realpath target/release/libchecklist_ffi.so)" ffi/bindings/python/
```

## Using the FFI Bindings

See [`cli.py`](./bindings/python/cli.py) for a usage example. Alternately, run the python CLI directly with

```sh
uv run ffi/bindings/python/cli.py -- --help
```

### Example

> [!IMPORTANT]
> This shows intended usage, and was copied (as `cli.py` was) from the [`libsql` spike](https://github.com/coriolinus/libsql-spike/).
>
> **THIS DOES NOT WORK**
>
> It turns out that Surrealdb, when run in embedded mode, requires that it is run within a Tokio runtime.
> This is incompatible with the Uniffi model of just using the target language's async runtime.

Note that neither CLI here is _good_ in the typical sense of the word; they were designed to be quick to implement and allow
a reasonable degree of control to show interop across FFI.

```sh
$ uv run ffi/bindings/python/cli.py -- create-checklist make python work
1
$ uv run ffi/bindings/python/cli.py -- create-item 1 finish ffi library
1
$ uv run ffi/bindings/python/cli.py -- create-item 1 create raw ffi bindings with uniffi
2
$ uv run ffi/bindings/python/cli.py -- create-item 1 demonstrate using the raw ffi bindings
3
$ uv run ffi/bindings/python/cli.py -- toggle-item 1
1
$ uv run ffi/bindings/python/cli.py -- toggle-item 2
1
$ uv run ffi/bindings/python/cli.py -- toggle-item 3
1
$ uv run ffi/bindings/python/cli.py -- create-item 1 show that this is the same db that rust can read
4
$ cargo run -p cli -- item show-all 1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/checklist item show-all 1`
     1: make python work
=========================
     1: ☑ finish ffi library
     2: ☑ create raw ffi bindings with uniffi
     3: ☑ demonstrate using the raw ffi bindings
     4: ☐ show that this is the same db that rust can read
$ uv run ffi/bindings/python/cli.py -- toggle-item 4
1
$ uv run ffi/bindings/python/cli.py -- create-item 1 show that this data is encrypted at rest
5
$ file ~/.local/share/checklist/db.sqlite3
Thu Feb 13 05:32:25 PM CET 2025
/home/coriolinus/.local/share/checklist/db.sqlite3: data
$ sqlite3 ~/.local/share/checklist/db.sqlite3
Thu Feb 13 05:33:27 PM CET 2025
SQLite version 3.45.1 2024-01-30 16:01:20
Enter ".help" for usage hints.
sqlite> .tables
Error: file is not a database
sqlite> .quit
$ strings ~/.local/share/checklist/db.sqlite3 | rg 'python|rust|ffi' || echo no comprehensible strings found
no comprehensible strings found
$ cargo run -p cli -- item show-all 1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/checklist item show-all 1`
     1: make python work
=========================
     1: ☑ finish ffi library
     2: ☑ create raw ffi bindings with uniffi
     3: ☑ demonstrate using the raw ffi bindings
     4: ☑ show that this is the same db that rust can read
     5: ☑ show that this data is encrypted at rest
```
