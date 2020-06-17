use std::cmp::Ordering;

use muta_codec_derive::RlpFixedCodec;
use serde::{Deserialize, Serialize};

use protocol::fixed_codec::{FixedCodec, FixedCodecError};
use protocol::types::{Address, Bytes, Hash, Metadata, ValidatorExtend};
use protocol::ProtocolResult;

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug)]
pub struct InitGenesisPayload {
    pub info:          GovernanceInfo,
    pub fee_address:   Address,
    pub miner_address: Address,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, Default)]
pub struct GovernanceInfo {
    pub admin:              Address,
    pub tx_failure_fee:     u64,
    pub tx_floor_fee:       u64,
    pub profit_deduct_rate: u64,
    pub tx_fee_discount:    Vec<DiscountLevel>,
    pub miner_benefit:      u64,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscountLevel {
    pub amount:               u64,
    pub discount_per_million: u64,
}

impl PartialOrd for DiscountLevel {
    fn partial_cmp(&self, other: &DiscountLevel) -> Option<Ordering> {
        self.amount.partial_cmp(&other.amount)
    }
}

impl Ord for DiscountLevel {
    fn cmp(&self, other: &DiscountLevel) -> Ordering {
        self.amount.cmp(&other.amount)
    }
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccmulateProfitPayload {
    pub address:           Address,
    pub accmulated_profit: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CalcFeePayload {
    pub profit: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransferFromPayload {
    pub asset_id:  Hash,
    pub sender:    Address,
    pub recipient: Address,
    pub value:     u64,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct Asset {
    pub id:        Hash,
    pub name:      String,
    pub symbol:    String,
    pub supply:    u64,
    pub precision: u64,
    pub issuer:    Address,
}
