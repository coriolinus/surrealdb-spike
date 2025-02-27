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

## Outcome

After completing Steps 1-5, it turns out that Surrealdb is built using Tokio-specific features and requires
that it is run within a Tokio runtime. This is strictly incompatible with Uniffi's model, in which Rust
futures are run within the target language's native async runtime.

This means that any attempt to actually use async Rust code will fail like this:

```text
$ uv run ffi/bindings/python/cli.py -- create-checklist show that python fails
Installed 2 packages in 28ms

thread '<unnamed>' panicked at ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/surrealdb-2.2.1/src/api/engine/local/native.rs:38:13:
there is no reactor running, must be called from the context of a Tokio 1.x runtime
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Traceback (most recent call last):
  File "~/projects/coriolinus/surrealdb-spike/ffi/bindings/python/cli.py", line 118, in <module>
    cli()
    ~~~^^
  File "~/.cache/uv/environments-v1/cli-4239edadcd61f658/lib/python3.13/site-packages/click/core.py", line 1161, in __call__
    return self.main(*args, **kwargs)
           ~~~~~~~~~^^^^^^^^^^^^^^^^^
  File "~/.cache/uv/environments-v1/cli-4239edadcd61f658/lib/python3.13/site-packages/click/core.py", line 1082, in main
    rv = self.invoke(ctx)
  File "~/.cache/uv/environments-v1/cli-4239edadcd61f658/lib/python3.13/site-packages/click/core.py", line 1694, in invoke
    super().invoke(ctx)
    ~~~~~~~~~~~~~~^^^^^
  File "~/.cache/uv/environments-v1/cli-4239edadcd61f658/lib/python3.13/site-packages/click/core.py", line 1443, in invoke
    return ctx.invoke(self.callback, **ctx.params)
           ~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "~/.cache/uv/environments-v1/cli-4239edadcd61f658/lib/python3.13/site-packages/click/core.py", line 788, in invoke
    return __callback(*args, **kwargs)
  File "~/.cache/uv/environments-v1/cli-4239edadcd61f658/lib/python3.13/site-packages/click/decorators.py", line 33, in new_func
    return f(get_current_context(), *args, **kwargs)
  File "~/projects/coriolinus/surrealdb-spike/ffi/bindings/python/cli.py", line 58, in cli
    ctx.obj = asyncio.run(checklist_ffi.db_new(path, key))
              ~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "~/.pyenv/versions/3.13.2/lib/python3.13/asyncio/runners.py", line 195, in run
    return runner.run(main)
           ~~~~~~~~~~^^^^^^
  File "~/.pyenv/versions/3.13.2/lib/python3.13/asyncio/runners.py", line 118, in run
    return self._loop.run_until_complete(task)
           ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^
  File "~/.pyenv/versions/3.13.2/lib/python3.13/asyncio/base_events.py", line 725, in run_until_complete
    return future.result()
           ~~~~~~~~~~~~~^^
  File "~/projects/coriolinus/surrealdb-spike/ffi/bindings/python/checklist_ffi.py", line 1869, in db_new
    return await _uniffi_rust_call_async(
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ...<11 lines>...
    )
    ^
  File "~/projects/coriolinus/surrealdb-spike/ffi/bindings/python/checklist_ffi.py", line 1784, in _uniffi_rust_call_async
    _uniffi_rust_call_with_error(error_ffi_converter, ffi_complete, rust_future)
    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "~/projects/coriolinus/surrealdb-spike/ffi/bindings/python/checklist_ffi.py", line 309, in _uniffi_rust_call_with_error
    _uniffi_check_call_status(error_ffi_converter, call_status)
    ~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "~/projects/coriolinus/surrealdb-spike/ffi/bindings/python/checklist_ffi.py", line 332, in _uniffi_check_call_status
    raise InternalError(msg)
checklist_ffi.InternalError: there is no reactor running, must be called from the context of a Tokio 1.x runtime
```

This completely rules out Surrealdb for our purposes.
