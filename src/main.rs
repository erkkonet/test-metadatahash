use codec::Decode;
use extension::{custom, CustomConfig};
use frame_metadata::RuntimeMetadataPrefixed;
use merkleized_metadata::ExtraInfo;
use std::fs;
use subxt::{config::DefaultExtrinsicParamsBuilder, tx::ValidationResult, OnlineClient};
use subxt_signer::sr25519::dev;

pub mod extension;

#[subxt::subxt(runtime_metadata_path = "artifacts/metadata")]
pub mod substrate {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<CustomConfig>::from_url("ws://127.0.0.1:9944").await?;

    let metadata: Vec<u8> = fs::read("artifacts/metadata")?;
    let runtime_metadata = RuntimeMetadataPrefixed::decode(&mut &metadata[..]).map(|x| x.1)?;
    let ss58prefix = api.constants().at(&substrate::constants().system().ss58_prefix())?;
    let version = api.constants().at(&substrate::constants().system().version())?;
    let extra_info = ExtraInfo {
        spec_version: version.spec_version,
        spec_name: version.spec_name,
        base58_prefix: ss58prefix,
        token_symbol: "Test".into(),
        decimals: 14,
    };

    let digest = merkleized_metadata::generate_metadata_digest(&runtime_metadata, extra_info)
        .unwrap()
        .hash();
    println!(
        "🔖   Metadata digest: {}",
        array_bytes::bytes2hex("0x", digest)
    );

    // Build a balance transfer extrinsic
    let dest = dev::bob().public_key().into();
    let balance_transfer_tx = substrate::tx()
        .balances()
        .transfer_allow_death(dest, 10_000);

    // Prepare the balance transfer extrinsic as Alice, using the custom extension config
    let tx_config = DefaultExtrinsicParamsBuilder::new();
    let from = dev::alice();
    let extrinsic = api
        .tx()
        .create_partial_signed(
            &balance_transfer_tx,
            &from.public_key().into(),
            custom(tx_config, digest),
        )
        .await?;

    // The payload can be validated in polkadot-js/apps under developer/extrinsics/decode
    // to make sure the right mode and metadatahash is displayed
    println!(
        "📝 Extrinsic Payload: {}",
        array_bytes::bytes2hex("0x", extrinsic.signer_payload())
    );

    let signed = extrinsic.sign(&from);

    // This can also be validated as above to check that the mode byte is 01
    println!(
        "🔏  Signed Extrinsic: {}",
        array_bytes::bytes2hex("0x", signed.encoded())
    );

    let result = signed.validate().await?;

    match result {
        ValidationResult::Valid(_) => println!("✅ Transaction is valid"),
        ValidationResult::Invalid(_) => println!("❌ Transaction is invalid"),
        ValidationResult::Unknown(_) => println!("❓ Transaction status is unknown"),
    }

    // signed.submit().await?;

    Ok(())
}
