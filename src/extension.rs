// based on subxt (https://github.com/paritytech/subxt) core/src/config/signed_extensions.rs
// with modifications to the CheckMetadataHash extension

// Copyright 2019-2024 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.

use codec::Encode;
use scale_decode::DecodeAsType;
use scale_encode::EncodeAsType;
use scale_info::PortableRegistry;
use subxt::{
    client::ClientState,
    config::{
        signed_extensions, DefaultExtrinsicParamsBuilder, ExtrinsicParams, ExtrinsicParamsEncoder,
        RefineParams, SignedExtension,
    },
    error::ExtrinsicParamsError,
    Config,
};

type Hash = [u8; 32];

#[derive(EncodeAsType)]
pub enum CustomConfig {}

impl Config for CustomConfig {
    type Hash = subxt::utils::H256;
    type AccountId = subxt::utils::AccountId32;
    type Address = subxt::utils::MultiAddress<Self::AccountId, ()>;
    type Signature = subxt::utils::MultiSignature;
    type Hasher = subxt::config::substrate::BlakeTwo256;
    type Header = subxt::config::substrate::SubstrateHeader<u32, Self::Hasher>;
    type ExtrinsicParams = signed_extensions::AnyOf<
        Self,
        (
            signed_extensions::CheckSpecVersion,
            signed_extensions::CheckTxVersion,
            signed_extensions::CheckNonce,
            signed_extensions::CheckGenesis<Self>,
            signed_extensions::CheckMortality<Self>,
            signed_extensions::ChargeAssetTxPayment<Self>,
            signed_extensions::ChargeTransactionPayment,
            CheckMetadataHash,
        ),
    >;
    type AssetId = u32;
}

/// The [`CheckMetadataHash`] signed extension.
pub struct CheckMetadataHash {
    pub mode: CheckMetadataHashMode,
    pub metadata_digest: Option<Hash>,
}

pub struct CheckMetadataHashParams<T: Config> {
    pub mode: CheckMetadataHashMode,
    pub metadata_digest: Option<Hash>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Config> Default for CheckMetadataHashParams<T> {
    fn default() -> Self {
        CheckMetadataHashParams {
            mode: CheckMetadataHashMode::Disabled,
            metadata_digest: None,
            _phantom: Default::default(),
        }
    }
}

impl<T: Config> CheckMetadataHashParams<T> {
    pub fn enabled(metadata_digest: Hash) -> Self {
        CheckMetadataHashParams {
            mode: CheckMetadataHashMode::Enabled,
            metadata_digest: Some(metadata_digest),
            _phantom: Default::default(),
        }
    }
}

impl<T: Config> RefineParams<T> for CheckMetadataHashParams<T> {}

impl<T: Config> ExtrinsicParams<T> for CheckMetadataHash {
    type Params = CheckMetadataHashParams<T>;

    fn new(_client: &ClientState<T>, params: Self::Params) -> Result<Self, ExtrinsicParamsError> {
        Ok(CheckMetadataHash {
            mode: params.mode,
            metadata_digest: params.metadata_digest,
        })
    }
}

impl ExtrinsicParamsEncoder for CheckMetadataHash {
    fn encode_extra_to(&self, v: &mut Vec<u8>) {
        self.mode.encode_to(v);
    }
    fn encode_additional_to(&self, v: &mut Vec<u8>) {
        self.mode.encode_to(v);
        if let Some(digest) = &self.metadata_digest {
            digest.encode_to(v);
        }
    }
}

impl<T: Config> SignedExtension<T> for CheckMetadataHash {
    type Decoded = CheckMetadataHashMode;
    fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
        identifier == "CheckMetadataHash"
    }
}

#[derive(Copy, Clone, Debug, DecodeAsType, Encode)]
pub enum CheckMetadataHashMode {
    /// No hash was provided in the signer payload.
    Disabled,
    /// A hash was provided in the signer payload.
    Enabled,
}

impl CheckMetadataHashMode {
    /// Is metadata checking enabled or disabled for this transaction?
    pub fn is_enabled(&self) -> bool {
        match self {
            CheckMetadataHashMode::Disabled => false,
            CheckMetadataHashMode::Enabled => true,
        }
    }
}

pub fn custom(
    params: DefaultExtrinsicParamsBuilder<CustomConfig>,
    digest: Hash,
) -> <<CustomConfig as Config>::ExtrinsicParams as ExtrinsicParams<CustomConfig>>::Params {
    let (a, b, c, d, e, f, g, _) = params.build();
    (a, b, c, d, e, f, g, CheckMetadataHashParams::enabled(digest))
}
