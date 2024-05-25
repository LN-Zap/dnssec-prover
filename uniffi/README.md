Build the bindings
```sh
uniffi-bindgen-cs src/interface.udl --config ./uniffi.toml
```

Build the dynamic library
```sh
cargo build --release
```

Check the dynamic library
```sh
nm -gU target/release/libdnssec_prover_uniffi.dylib
```