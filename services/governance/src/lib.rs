#[cfg(test)]
mod tests;
mod types;

use std::cell::RefCell;
use std::rc::Rc;

use bytes::Bytes;
use derive_more::{Display, From};
use serde::Serialize;

use binding_macro::{cycles, genesis, hook_after, service, tx_hook_after};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap};
use protocol::types::{Address, Metadata, ServiceContext, ServiceContextParams};

use crate::types::{
    AccmulateProfitPayload, Asset, CalcFeePayload, DiscountLevel, GovernanceInfo,
    InitGenesisPayload, SetAdminEvent, SetAdminPayload, SetGovernInfoEvent, SetGovernInfoPayload,
    TransferFromPayload, UpdateIntervalEvent, UpdateIntervalPayload, UpdateMetadataEvent,
    UpdateMetadataPayload, UpdateRatioEvent, UpdateRatioPayload, UpdateValidatorsEvent,
    UpdateValidatorsPayload,
};

const ADMIN_KEY: &str = "admin";
const FEE_ADDRESS_KEY: &str = "fee_addrss";
const MINER_ADDRESS_KEY: &str = "miner_address";
const MILLION: u64 = 1_000_000;
const HUNDRED: u64 = 100;
static ADMISSION_TOKEN: Bytes = Bytes::from_static(b"governance");

pub struct GovernanceService<SDK> {
    sdk:     SDK,
    profits: Box<dyn StoreMap<Address, u64>>,
}

#[service]
impl<SDK: ServiceSDK> GovernanceService<SDK> {
    pub fn new(mut sdk: SDK) -> Self {
        let profits: Box<dyn StoreMap<Address, u64>> = sdk.alloc_or_recover_map("profit");
        Self { sdk, profits }
    }

    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) {
        assert!(self.profits.is_empty());

        let mut info = payload.info;
        info.tx_fee_discount.sort();
        self.sdk.set_value(ADMIN_KEY.to_string(), info);
        self.sdk
            .set_value(FEE_ADDRESS_KEY.to_string(), payload.fee_address);
        self.sdk
            .set_value(MINER_ADDRESS_KEY.to_string(), payload.miner_address);
    }

    #[cycles(210_00)]
    #[read]
    fn get_admin_address(&self, ctx: ServiceContext) -> ServiceResponse<Address> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info.admin)
    }

    #[cycles(210_00)]
    #[read]
    fn get_govern_info(&self, ctx: ServiceContext) -> ServiceResponse<GovernanceInfo> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info)
    }

    #[cycles(210_00)]
    #[read]
    fn get_tx_failure_fee(&self, ctx: ServiceContext) -> ServiceResponse<u64> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info.tx_failure_fee)
    }

    #[cycles(210_00)]
    #[read]
    fn get_tx_floor_fee(&self, ctx: ServiceContext) -> ServiceResponse<u64> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info.tx_floor_fee)
    }

    #[cycles(210_00)]
    #[write]
    fn set_admin(&mut self, ctx: ServiceContext, payload: SetAdminPayload) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");
        info.admin = payload.admin.clone();

        self.sdk.set_value(ADMIN_KEY.to_owned(), info);

        let event = SetAdminEvent {
            topic: "Set New Admin".to_owned(),
            admin: payload.admin,
        };
        Self::emit_event(&ctx, event)
    }

    #[cycles(210_00)]
    #[write]
    fn set_govern_info(
        &mut self,
        ctx: ServiceContext,
        payload: SetGovernInfoPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut info = payload.inner;
        info.tx_fee_discount.sort();
        self.sdk.set_value(ADMIN_KEY.to_owned(), info.clone());

        let event = SetGovernInfoEvent {
            topic: "Set New Govern Info".to_owned(),
            info,
        };
        Self::emit_event(&ctx, event)
    }

    #[cycles(210_00)]
    #[write]
    fn update_metadata(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        if let Err(err) = self.write_metadata(&ctx, payload.clone()) {
            return err;
        }

        Self::emit_event(&ctx, UpdateMetadataEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn update_validators(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateValidatorsPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };

        metadata.verifier_list = payload.verifier_list.clone();
        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(&ctx, UpdateValidatorsEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn update_interval(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateIntervalPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };

        metadata.interval = payload.interval;
        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(&ctx, UpdateIntervalEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn update_ratio(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateRatioPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };
        metadata.propose_ratio = payload.propose_ratio;
        metadata.prevote_ratio = payload.prevote_ratio;
        metadata.precommit_ratio = payload.precommit_ratio;
        metadata.brake_ratio = payload.brake_ratio;

        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(&ctx, UpdateRatioEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn accumulate_profit(
        &mut self,
        ctx: ServiceContext,
        payload: AccmulateProfitPayload,
    ) -> ServiceResponse<()> {
        let address = payload.address;
        let new_profit = payload.accumulated_profit;

        if let Some(profit) = self.profits.get(&address) {
            if let Some(profit_sum) = profit.checked_add(new_profit) {
                self.profits.insert(address, profit_sum);
            } else {
                return ServiceResponse::from_error(101, "profit overflow".to_owned());
            }
        } else {
            self.profits.insert(address, new_profit);
        }

        ServiceResponse::from_succeed(())
    }

    #[cycles(210_00)]
    #[read]
    fn calc_tx_fee(&self, ctx: ServiceContext, payload: CalcFeePayload) -> ServiceResponse<u64> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");

        if let Some(tmp) = payload.profit.checked_mul(info.profit_deduct_rate) {
            if let Some(tmp_fee) = self.calc_discount_fee(tmp / MILLION, &info.tx_fee_discount) {
                return ServiceResponse::from_succeed(tmp_fee.max(info.tx_floor_fee));
            }
        }

        ServiceResponse::from_error(101, "fee overflow".to_owned())
    }

    #[tx_hook_after]
    fn tx_hook_after_(&mut self, ctx: ServiceContext) {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");
        let fee_address: Address = self.sdk.get_value(&FEE_ADDRESS_KEY.to_owned()).unwrap();
        let profit_deduct_rate = info.profit_deduct_rate;
        let asset = self
            .get_native_asset(&ctx)
            .expect("Can not get native asset");
        let profits = self
            .profits
            .iter()
            .map(|i| (i.0.clone(), i.1))
            .collect::<Vec<_>>();

        for (addr, profit) in profits.iter() {
            let tmp_fee = if let Some(fee) = profit.checked_mul(profit_deduct_rate) {
                fee
            } else {
                continue;
            };

            if let Some(fee) = self.calc_discount_fee(tmp_fee, &info.tx_fee_discount) {
                let _ = self.transfer_from(&ctx, TransferFromPayload {
                    asset_id:  asset.id.clone(),
                    sender:    addr.clone(),
                    recipient: fee_address.clone(),
                    value:     fee.max(info.tx_floor_fee),
                });
            }
        }
    }

    #[hook_after]
    fn hook_after_(&mut self, params: &ExecutorParams) {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");
        let sender_address: Address = self
            .sdk
            .get_value(&MINER_ADDRESS_KEY.to_owned())
            .expect("send miner fee address should not be none");

        let ctx_params = ServiceContextParams {
            tx_hash:         None,
            nonce:           None,
            cycles_limit:    params.cycles_limit,
            cycles_price:    1,
            cycles_used:     Rc::new(RefCell::new(0)),
            caller:          sender_address.clone(),
            height:          params.height,
            service_name:    String::new(),
            service_method:  String::new(),
            service_payload: String::new(),
            extra:           None,
            timestamp:       params.timestamp,
            events:          Rc::new(RefCell::new(vec![])),
        };

        let ctx = ServiceContext::new(ctx_params);
        let asset = self
            .get_native_asset(&ctx)
            .expect("Can not get native asset");

        let payload = TransferFromPayload {
            asset_id:  asset.id,
            sender:    sender_address,
            recipient: params.proposer.clone(),
            value:     info.miner_benefit,
        };

        let _ = self.transfer_from(&ctx, payload);
    }

    fn calc_discount_fee(&self, origin_fee: u64, discount_level: &[DiscountLevel]) -> Option<u64> {
        let mut discount = HUNDRED;
        for level in discount_level.iter().rev() {
            if origin_fee >= level.amount {
                discount = level.discount_per_million;
                break;
            }
        }

        let res = origin_fee.checked_mul(discount)?;
        Some(res / HUNDRED)
    }

    fn is_admin(&self, ctx: &ServiceContext) -> bool {
        let caller = ctx.get_caller();
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_string())
            .expect("Admin should not be none");

        info.admin == caller
    }

    fn get_metadata(&self, ctx: &ServiceContext) -> Result<Metadata, ServiceResponse<()>> {
        let resp = self.sdk.read(ctx, None, "metadata", "get_metadata", "");
        if resp.is_error() {
            return Err(ServiceResponse::from_error(resp.code, resp.error_message));
        }

        let meta_json: String = resp.succeed_data;
        let meta = serde_json::from_str(&meta_json).map_err(ServiceError::JsonParse)?;
        Ok(meta)
    }

    fn write_metadata(
        &mut self,
        ctx: &ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> Result<(), ServiceResponse<()>> {
        let payload_json = match serde_json::to_string(&payload) {
            Ok(j) => j,
            Err(err) => return Err(ServiceError::JsonParse(err).into()),
        };

        let resp = self.sdk.write(
            &ctx,
            Some(ADMISSION_TOKEN.clone()),
            "metadata",
            "update_metadata",
            &payload_json,
        );

        if resp.is_error() {
            Err(ServiceResponse::from_error(resp.code, resp.error_message))
        } else {
            Ok(())
        }
    }

    fn transfer_from(
        &mut self,
        ctx: &ServiceContext,
        payload: TransferFromPayload,
    ) -> Result<(), ServiceResponse<()>> {
        let payload_json = match serde_json::to_string(&payload) {
            Ok(j) => j,
            Err(err) => return Err(ServiceError::JsonParse(err).into()),
        };

        let resp = self.sdk.write(
            &ctx,
            Some(ADMISSION_TOKEN.clone()),
            "asset",
            "transfer_from",
            &payload_json,
        );

        if resp.is_error() {
            Err(ServiceResponse::from_error(resp.code, resp.error_message))
        } else {
            Ok(())
        }
    }

    fn get_native_asset(&self, ctx: &ServiceContext) -> Result<Asset, ServiceResponse<Asset>> {
        let resp = self.sdk.read(
            &ctx,
            Some(ADMISSION_TOKEN.clone()),
            "asset",
            "get_native_asset",
            "",
        );

        if resp.is_error() {
            Err(ServiceResponse::from_error(resp.code, resp.error_message))
        } else {
            let ret: Asset = serde_json::from_str(&resp.succeed_data).unwrap();
            Ok(ret)
        }
    }

    fn emit_event<T: Serialize>(ctx: &ServiceContext, event: T) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::JsonParse(err).into(),
            Ok(json) => {
                ctx.emit_event(json);
                ServiceResponse::from_succeed(())
            }
        }
    }

    #[cfg(test)]
    pub fn get_fee(&self, address: &Address) -> Option<u64> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("Admin should not be none");

        let profit = if let Some(tmp) = self.profits.get(address) {
            tmp
        } else {
            return None;
        };

        if let Some(tmp) = profit.checked_mul(info.profit_deduct_rate) {
            if let Some(tmp_fee) = self.calc_discount_fee(tmp / MILLION, &info.tx_fee_discount) {
                return Some(tmp_fee.max(info.tx_floor_fee));
            }
        }
        None
    }
}

#[derive(Debug, Display, From)]
pub enum ServiceError {
    NonAuthorized,

    #[display(fmt = "Parsing payload to json failed {:?}", _0)]
    JsonParse(serde_json::Error),
}

impl ServiceError {
    fn code(&self) -> u64 {
        match self {
            ServiceError::NonAuthorized => 101,
            ServiceError::JsonParse(_) => 102,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
