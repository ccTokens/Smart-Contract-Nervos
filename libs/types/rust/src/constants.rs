#[cfg(feature = "no_std")]
use alloc::vec::Vec;
use core::cell::OnceCell;
use core::convert::TryFrom;
use core::env;

#[cfg(feature = "no_std")]
use ckb_std::ckb_constants::Source as CkbSource;
#[cfg(feature = "no_std")]
use ckb_std::ckb_types::core::ScriptHashType;
#[cfg(feature = "no_std")]
use ckb_std::ckb_types::packed::Byte;
#[cfg(feature = "no_std")]
use ckb_std::debug;
#[cfg(not(feature = "no_std"))]
use ckb_types::core::ScriptHashType;
#[cfg(not(feature = "no_std"))]
use ckb_types::packed::Byte;
use molecule::prelude::{Builder, Entity};
use num_enum::TryFromPrimitive;
use strum::{Display, EnumString};

use super::schemas::packed::{self, Byte32, Script};

pub const CKB_HASH_DIGEST: usize = 32;
pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub const OMNI_FLAG_MULTISIG: u8 = 6;
pub const OMNI_FLAG_NO_MODE: u8 = 0;

pub const TOKEN_ID_SIZE: usize = 32;
pub const XUDT_OWNER_LOCK_HASH_SIZE: usize = 32;
pub const XUDT_TYPE_ARGS_FLAG_SIZE: usize = 4;

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum SystemStatus {
    Off,
    On,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum Source {
    Input = 1,
    Output = 2,
    CellDep = 3,
}

#[cfg(feature = "no_std")]
impl From<CkbSource> for Source {
    fn from(source: CkbSource) -> Self {
        match source {
            CkbSource::Input => Source::Input,
            CkbSource::Output => Source::Output,
            CkbSource::CellDep => Source::CellDep,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "no_std")]
impl Into<CkbSource> for Source {
    fn into(self) -> CkbSource {
        match self {
            Source::Input => CkbSource::Input,
            Source::Output => CkbSource::Output,
            Source::CellDep => CkbSource::CellDep,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, EnumString, Display)]
pub enum Action {
    #[strum(serialize = "deploy_config")]
    DeployConfig,
    #[strum(serialize = "update_config")]
    UpdateConfig,
    #[strum(serialize = "init_governance")]
    InitGovernance,
    #[strum(serialize = "update_owner")]
    UpdateOwner,
    #[strum(serialize = "update_custodians")]
    UpdateCustodians,
    #[strum(serialize = "update_merchants")]
    UpdateMerchants,
    #[strum(serialize = "deploy_token")]
    DeployToken,
    #[strum(serialize = "request_mint")]
    RequestMint,
    #[strum(serialize = "confirm_mint")]
    ConfirmMint,
    #[strum(serialize = "reject_mint")]
    RejectMint,
    #[strum(serialize = "request_burn")]
    RequestBurn,
    #[strum(serialize = "confirm_burn")]
    ConfirmBurn,
    #[strum(serialize = "reject_burn")]
    RejectBurn,
    #[default]
    Others,
}

impl Action {
    pub fn new(action_str: &str) -> Self {
        action_str.parse::<Action>().unwrap_or_else(|_e| {
            #[cfg(feature = "no_std")]
            debug!("Failed to convert string to Action, error:{:?}", _e);
            Action::Others
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, EnumString, Display, TryFromPrimitive)]
#[repr(u8)]
pub enum GovernanceMemberRole {
    #[strum(serialize = "custodian")]
    Custodian,
    #[strum(serialize = "merchant")]
    Merchant,
}

#[derive(Clone, Copy, Debug, PartialEq, EnumString, Display, TryFromPrimitive)]
#[repr(u32)]
pub enum ConfigKey {
    #[strum(serialize = "system_status")]
    SystemStatus,
    #[strum(serialize = "governance_member_cell_type_id")]
    GovernanceMemberCellTypeId,
    #[strum(serialize = "governance_member_cell_type_args")]
    GovernanceMemberCellTypeArgs,
    #[strum(serialize = "tick_cell_type_id")]
    TickCellTypeId,
    #[strum(serialize = "tick_cell_type_args")]
    TickCellTypeArgs,
    #[strum(serialize = "xudt_info_cell_type_id")]
    XudtInfoCellTypeId,
    #[strum(serialize = "xudt_info_cell_type_args")]
    XudtInfoCellTypeArgs,
    #[strum(serialize = "xudt_cell_type_id")]
    XudtCellTypeId,
    #[strum(serialize = "xudt_cell_type_args")]
    XudtCellTypeArgs,
    #[strum(serialize = "always_success_type_id")]
    AlwaysSuccessTypeId,
    #[strum(serialize = "always_success_type_args")]
    AlwaysSuccessTypeArgs,
    #[strum(serialize = "omni_lock_type_id")]
    OmniLockTypeId,
    #[strum(serialize = "omni_lock_type_args")]
    OmniLockTypeArgs,
    // #[strum(serialize = "xudt_extension_type_id")]
    // XudtExtensionTypeId,
    // #[strum(serialize = "xudt_extension_type_args")]
    // XudtExtensionTypeArgs,
    #[strum(serialize = "xudt_owner_type_id")]
    XudtOwnerTypeId = 15,
    #[strum(serialize = "xudt_owner_type_args")]
    XudtOwnerTypeArgs,
    #[strum(serialize = "xudt_info_cell_type_out_point")]
    XudtInfoCellTypeOutPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, EnumString, Display, TryFromPrimitive)]
#[repr(u8)]
pub enum TickType {
    #[strum(serialize = "mint")]
    Mint = 0,
    #[strum(serialize = "burn")]
    Burn = 1,
}

pub fn deploy_lock() -> &'static Script {
    static mut DEPLOY_LOCK: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("DEPLOY_CODE_HASH").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The DEPLOY_CODE_HASH should be a hex string.");

    let args = env!("DEPLOY_ARGS").trim_start_matches("0x");
    let args = hex::decode(args).expect("The DEPLOY_ARGS should be a hex string.");

    unsafe {
        DEPLOY_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Byte32::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn owner_lock() -> &'static Script {
    static mut OWNER_LOCK: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("OWNER_CODE_HASH").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The OWNER_CODE_HASH should be a hex string.");

    let args = env!("OWNER_ARGS").trim_start_matches("0x");
    let args = hex::decode(args).expect("The OWNER_ARGS should be a hex string.");

    unsafe {
        OWNER_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Byte32::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn config_cell_type_id() -> Vec<u8> {
    let type_id = env!("CONFIG_CELL_TYPE_ID").trim_start_matches("0x");
    hex::decode(type_id).expect("The CONFIG_CELL_TYPE_ID should be a hex string.")
}

pub fn config_cell_type() -> &'static Script {
    static mut CONFIG_CELL_TYPE: OnceCell<Script> = OnceCell::new();
    let type_id = config_cell_type_id();

    unsafe {
        CONFIG_CELL_TYPE.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Byte32::try_from(type_id).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .build();
            script
        })
    }
}
