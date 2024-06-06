# test-metadatahash

Simple example using [subxt](https://github.com/paritytech/subxt) to test the checkmetadata-hash extension in a runtime.

First download the metadata for your runtime (needs `subxt-cli` installed):

```shell
subxt metadata -f bytes --url ws://127.0.0.1:9944 > artifacts/metadata
```

then adjust the symbol and decimals in `extra_info` in [main.rs](src/main.rs) to match your runtime:

```rust
    let extra_info = ExtraInfo {
        ...
        token_symbol: "Test".into(),
        decimals: 14,
    };
```
> The token_symbol and decimals should match the runtime wasm-builder `enable_metadata_hash(token_symbol, decimals)` 
arguments, which may differ from the chainspec properties.

You might want to change the test extrinsic (a Balances transfer from `//Alice` to `//Bob`). Then run:

```shell
cargo run
```

Hopefully you will see something like:
```shell
🔖   Metadata digest: 0x228f8af55de99defd4dedda785eda17ed70ecb4f4ca9bfc18f114631f1a3421f
📝 Extrinsic Payload: 0x0600008eaf04151687736326c9fea1..d70ecb4f4ca9bfc18f114631f1a3421f
🔏  Signed Extrinsic: 0x35028400d43593c715fdd31c61141a..3693c912909cb226aa4794f26a48419c
✅ Transaction is valid
```

For troubleshooting, you can find the runtime metadata digest by running your node with 
`RUST_LOG=runtime::metadata-hash=debug` and looking for the following log when submitting an extrinsic:

```shell
DEBUG tokio-runtime-worker runtime::metadata-hash: CheckMetadataHash::additional_signed 
  => Some("0x228f8af55de99defd4dedda785eda17ed70ecb4f4ca9bfc18f114631f1a3421f")
```
