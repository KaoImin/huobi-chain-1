#[cfg(test)]
mod tests;
mod types;

use bytes::Bytes;
use derive_more::{Display, From};
use serde::Serialize;

use binding_macro::{cycles, genesis, service};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK};
use protocol::types::{Address, Metadata, ServiceContext};

use crate::types::{
    GovernanceInfo, InitGenesisPayload, SetAdminEvent, SetAdminPayload, SetGovernInfoEvent,
    SetGovernInfoPayload, UpdateIntervalEvent, UpdateIntervalPayload, UpdateMetadataEvent,
    UpdateMetadataPayload, UpdateRatioEvent, UpdateRatioPayload, UpdateValidatorsEvent,
    UpdateValidatorsPayload,
};

const ADMIN_KEY: &str = "admin";
static ADMISSION_TOKEN: Bytes = Bytes::from_static(b"governance");

pub struct GovernanceService<SDK> {
    sdk: SDK,
}

#[service]
impl<SDK: ServiceSDK> GovernanceService<SDK> {
    pub fn new(sdk: SDK) -> Self {
        Self { sdk }
    }

    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) {
        self.sdk.set_value(ADMIN_KEY.to_string(), payload.inner);
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

        self.sdk
            .set_value(ADMIN_KEY.to_owned(), payload.admin.clone());

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

        self.sdk
            .set_value(ADMIN_KEY.to_owned(), payload.inner.clone());

        let event = SetGovernInfoEvent {
            topic: "Set New Govern Info".to_owned(),
            info:  payload.inner,
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

        let event = UpdateMetadataEvent::from(payload);
        Self::emit_event(&ctx, event)
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

        let event = UpdateValidatorsEvent {
            topic:         "Validators Updated".to_owned(),
            verifier_list: payload.verifier_list,
        };
        Self::emit_event(&ctx, event)
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

        let event = UpdateIntervalEvent {
            topic:    "Interval Updated".to_owned(),
            interval: payload.interval,
        };
        Self::emit_event(&ctx, event)
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

        let metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };

        let update_metadata_payload = UpdateMetadataPayload {
            verifier_list:   metadata.verifier_list,
            interval:        metadata.interval,
            propose_ratio:   payload.propose_ratio,
            prevote_ratio:   payload.prevote_ratio,
            precommit_ratio: payload.precommit_ratio,
            brake_ratio:     payload.brake_ratio,
        };
        if let Err(err) = self.write_metadata(&ctx, update_metadata_payload) {
            return err;
        }

        let event = UpdateRatioEvent {
            topic:           "Ratio Updated".to_owned(),
            propose_ratio:   payload.propose_ratio,
            prevote_ratio:   payload.prevote_ratio,
            precommit_ratio: payload.precommit_ratio,
            brake_ratio:     payload.brake_ratio,
        };
        Self::emit_event(&ctx, event)
    }

    fn is_admin(&self, ctx: &ServiceContext) -> bool {
        let caller = ctx.get_caller();
        let admin: Address = self
            .sdk
            .get_value(&ADMIN_KEY.to_string())
            .expect("Admin should not be none");

        admin == caller
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

    fn emit_event<T: Serialize>(ctx: &ServiceContext, event: T) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::JsonParse(err).into(),
            Ok(json) => {
                ctx.emit_event(json);
                ServiceResponse::from_succeed(())
            }
        }
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
