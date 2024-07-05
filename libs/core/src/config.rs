use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use core::cell::OnceCell;
use core::convert::TryFrom;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::core::ScriptHashType;
use ckb_std::high_level;
use types::constants::{config_cell_type_id, ConfigKey, SystemStatus};
use types::packed::{Byte, Byte32, Script};
use types::prelude::{Builder, Entity};

use super::error::CoreError;
use crate::constants::ScriptType;
use crate::data_parser::config_cell;
use crate::util;

#[derive(Debug)]
pub struct Config {
    pub inited: OnceCell<bool>,
    // Config fields
    pub system_status: SystemStatus,
    pub governance_member_cell_type_id: Vec<u8>,
    pub governance_member_cell_type_args: Vec<u8>,
    pub tick_cell_type_id: Vec<u8>,
    pub tick_cell_type_args: Vec<u8>,
    pub xudt_info_cell_type_id: Vec<u8>,
    pub xudt_info_cell_type_args: Vec<u8>,
    pub xudt_cell_type_id: Vec<u8>,
    pub xudt_cell_type_args: Vec<u8>,
    pub always_success_type_id: Vec<u8>,
    pub always_success_type_args: Vec<u8>,
    pub omni_lock_type_id: Vec<u8>,
    pub omni_lock_type_args: Vec<u8>,
    // pub xudt_extension_type_id: Vec<u8>,
    // pub xudt_extension_type_args: Vec<u8>,
    pub xudt_owner_type_id: Vec<u8>,
    pub xudt_owner_type_args: Vec<u8>,
    pub xudt_info_cell_out_point: Vec<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inited: OnceCell::new(),
            system_status: SystemStatus::On,
            governance_member_cell_type_id: vec![0u8; 32],
            governance_member_cell_type_args: vec![0u8; 32],
            tick_cell_type_id: vec![0u8; 32],
            tick_cell_type_args: vec![0u8; 32],
            xudt_info_cell_type_id: vec![0u8; 32],
            xudt_info_cell_type_args: vec![0u8; 32],
            xudt_cell_type_id: vec![0u8; 32],
            xudt_cell_type_args: vec![0u8; 32],
            always_success_type_id: vec![0u8; 32],
            always_success_type_args: vec![0u8; 32],
            omni_lock_type_id: vec![0u8; 32],
            omni_lock_type_args: vec![0u8; 32],
            // xudt_extension_type_id: vec![0u8; 32],
            // xudt_extension_type_args: vec![0u8; 32],
            xudt_owner_type_id: vec![0u8; 32],
            xudt_owner_type_args: vec![0u8; 32],
            xudt_info_cell_out_point: vec![0u8; 33],
        }
    }
}

impl Config {
    pub fn get_instance() -> &'static mut Self {
        static mut CONFIG: OnceCell<Config> = OnceCell::new();
        unsafe {
            CONFIG.get_or_init(|| {
                let res = Self::default();
                res
            });
            CONFIG.get_mut().unwrap()
        }
    }

    pub fn load_data_from_cell(&mut self) -> Result<(), CoreError> {
        let type_id = Byte32::try_from(config_cell_type_id()).map_err(|_| CoreError::DotEnvError)?;
        let index =
            util::find_only_cell_by_type_id("ConfigCell", ScriptType::Type, type_id.as_reader(), Source::CellDep)?;
        let data = high_level::load_cell_data(index, Source::CellDep).map_err(CoreError::from)?;
        let (_version, configs) = config_cell::parse(&data)?;

        for (key, value) in configs {
            match key {
                ConfigKey::SystemStatus => {
                    let status = SystemStatus::try_from(value[0]).map_err(|_| CoreError::ParseCellDataFailed {
                        cell_name: String::from("ConfigCell"),
                        msg: format!("Can not parse value to SystemStatus."),
                    })?;
                    self.system_status = status;
                }
                ConfigKey::GovernanceMemberCellTypeId => {
                    self.governance_member_cell_type_id = value;
                }
                ConfigKey::GovernanceMemberCellTypeArgs => {
                    self.governance_member_cell_type_args = value;
                }
                ConfigKey::TickCellTypeId => {
                    self.tick_cell_type_id = value;
                }
                ConfigKey::TickCellTypeArgs => {
                    self.tick_cell_type_args = value;
                }
                ConfigKey::XudtInfoCellTypeId => {
                    self.xudt_info_cell_type_id = value;
                }
                ConfigKey::XudtInfoCellTypeArgs => {
                    self.xudt_info_cell_type_args = value;
                }
                ConfigKey::XudtCellTypeId => {
                    self.xudt_cell_type_id = value;
                }
                ConfigKey::XudtCellTypeArgs => {
                    self.xudt_cell_type_args = value;
                }
                ConfigKey::AlwaysSuccessTypeId => {
                    self.always_success_type_id = value;
                }
                ConfigKey::AlwaysSuccessTypeArgs => {
                    self.always_success_type_args = value;
                }
                ConfigKey::OmniLockTypeId => {
                    self.omni_lock_type_id = value;
                }
                ConfigKey::OmniLockTypeArgs => {
                    self.omni_lock_type_args = value;
                }
                ConfigKey::XudtOwnerTypeId => {
                    self.xudt_owner_type_id = value;
                }
                ConfigKey::XudtOwnerTypeArgs => {
                    self.xudt_owner_type_args = value;
                }
                ConfigKey::XudtInfoCellTypeOutPoint => {
                    self.xudt_info_cell_out_point = value;
                }
            }
        }

        Ok(())
    }
}

//check if the system is on
pub fn check_system_status() -> Result<(), CoreError> {
    let config = Config::get_instance();
    config.load_data_from_cell().map_err(|e| {
        warn!("{}", e.to_string());
        e
    })?;

    if config.system_status == SystemStatus::Off {
        return Err(CoreError::SystemStatusOff);
    }

    Ok(())
}

pub fn get_config_by_key(key: ConfigKey) -> Result<Vec<u8>, CoreError> {
    let config = Config::get_instance();
    config.load_data_from_cell().map_err(|e| {
        warn!("{}", e.to_string());
        e
    })?;

    match key {
        ConfigKey::SystemStatus => Ok(vec![config.system_status as u8]),
        ConfigKey::GovernanceMemberCellTypeId => Ok(config.governance_member_cell_type_id.clone()),
        ConfigKey::GovernanceMemberCellTypeArgs => Ok(config.governance_member_cell_type_args.clone()),
        ConfigKey::TickCellTypeId => Ok(config.tick_cell_type_id.clone()),
        ConfigKey::TickCellTypeArgs => Ok(config.tick_cell_type_args.clone()),
        ConfigKey::XudtInfoCellTypeId => Ok(config.xudt_info_cell_type_id.clone()),
        ConfigKey::XudtInfoCellTypeArgs => Ok(config.xudt_info_cell_type_args.clone()),
        ConfigKey::XudtCellTypeId => Ok(config.xudt_cell_type_id.clone()),
        ConfigKey::XudtCellTypeArgs => Ok(config.xudt_cell_type_args.clone()),
        ConfigKey::AlwaysSuccessTypeId => Ok(config.always_success_type_id.clone()),
        ConfigKey::AlwaysSuccessTypeArgs => Ok(config.always_success_type_args.clone()),
        ConfigKey::OmniLockTypeId => Ok(config.omni_lock_type_id.clone()),
        ConfigKey::OmniLockTypeArgs => Ok(config.omni_lock_type_args.clone()),
        ConfigKey::XudtOwnerTypeId => Ok(config.xudt_owner_type_id.clone()),
        ConfigKey::XudtOwnerTypeArgs => Ok(config.xudt_owner_type_args.clone()),
        ConfigKey::XudtInfoCellTypeOutPoint => Ok(config.xudt_info_cell_out_point.clone()),
    }
}

pub fn always_success_lock() -> Result<Script, CoreError> {
    let code_hash = get_config_by_key(ConfigKey::AlwaysSuccessTypeId)?;

    Ok(Script::new_builder()
        .code_hash(Byte32::try_from(code_hash).unwrap())
        .hash_type(Byte::new(ScriptHashType::Type.into()))
        .build())
}

pub fn omni_lock_type_id() -> Result<Vec<u8>, CoreError> {
    get_config_by_key(ConfigKey::OmniLockTypeId)
}

pub fn governance_member_cell_type_id() -> Result<Vec<u8>, CoreError> {
    get_config_by_key(ConfigKey::GovernanceMemberCellTypeId)
}
