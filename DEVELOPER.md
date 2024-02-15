# Developer Guide for ExtendR

## Debugging Rust Macros

In `.cargo/config.toml`, you can place

```toml
[build]
rustflags = [
    "-Zmacro-backtrace",
    "-Zdebug-macros",
    "-Zproc-macro-backtrace",
    # "-Ztrace-macros", # very verbose option
]
```

To get verbose diagnostics on macros. These are _nightly_ features,
thus the compiler needs to be set to `nightly`, either by `cargo +nightly`,
or even `rustup default nightly`. Alternatively, a `rust-toolchain.toml`
file can be used to set a workspace requirement.
