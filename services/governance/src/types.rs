use muta_codec_derive::RlpFixedCodec;
use serde::{Deserialize, Serialize};

use protocol::fixed_codec::{FixedCodec, FixedCodecError};
use protocol::types::{Address, Bytes, Metadata, ValidatorExtend};
use protocol::ProtocolResult;

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug)]
pub struct InitGenesisPayload {
    pub inner: GovernanceInfo,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, Default)]
pub struct GovernanceInfo {
    pub admin:              Address,
    pub tx_failure_fee:     u64,
    pub tx_floor_fee:       u64,
    pub profit_deduct_rate: u64,
    pub tx_fee_discount:    DiscountLevel,
    pub miner_benefit:      u64,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, Default)]
pub struct DiscountLevel {
    pub amount:               u64,
    pub discount_pre_million: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetAdminPayload {
    pub admin: Address,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetGovernInfoPayload {
    pub inner: GovernanceInfo,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetAdminEvent {
    pub topic: String,
    pub admin: Address,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetGovernInfoEvent {
    pub topic: String,
    pub info:  GovernanceInfo,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateMetadataPayload {
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}

impl From<Metadata> for UpdateMetadataPayload {
    fn from(metadata: Metadata) -> Self {
        UpdateMetadataPayload {
            verifier_list:   metadata.verifier_list,
            interval:        metadata.interval,
            propose_ratio:   metadata.propose_ratio,
            prevote_ratio:   metadata.prevote_ratio,
            precommit_ratio: metadata.precommit_ratio,
            brake_ratio:     metadata.brake_ratio,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateMetadataEvent {
    pub topic:           String,
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}

impl From<UpdateMetadataPayload> for UpdateMetadataEvent {
    fn from(payload: UpdateMetadataPayload) -> Self {
        UpdateMetadataEvent {
            topic:           "Metadata Updated".to_owned(),
            verifier_list:   payload.verifier_list,
            interval:        payload.interval,
            propose_ratio:   payload.propose_ratio,
            prevote_ratio:   payload.prevote_ratio,
            precommit_ratio: payload.precommit_ratio,
            brake_ratio:     payload.brake_ratio,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateValidatorsPayload {
    pub verifier_list: Vec<ValidatorExtend>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateValidatorsEvent {
    pub topic:         String,
    pub verifier_list: Vec<ValidatorExtend>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateIntervalPayload {
    pub interval: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateIntervalEvent {
    pub topic:    String,
    pub interval: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateRatioPayload {
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateRatioEvent {
    pub topic:           String,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}
