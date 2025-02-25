# `surrealdb` Spike

We are looking for a unified database solution which

- is async
- builds to wasm
- works via uniffi

[`surrealdb`] looks like it may meet our criteria, so let's prove that concept one way or the other.

[`surrealdb`]: https://surrealdb.com/features

## Strategy

1. Build a simple pure-rust library using this DB
2. Build a simple CLI so we can prove it works locally
3. Build a -ffi adaptor crate
4. Add `uniffi` bindings to the ffi adaptor crate.
5. Verify that some other language i.e. python can access the things uniffi exports
6. Add `wasm` bindings to the ffi adaptor crate.
7. Verify that TS in bun can access the things wasm exports.

The FFI stuff is complicated, but we can streamline things by essentially copying config from core-crypto.
The point of all this is to demonstrate the capability on a project with a (much!) smaller surface area
than core-crypto itself.
