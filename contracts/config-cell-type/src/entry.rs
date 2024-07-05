use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::{debug, high_level};
use contract_core::constants::ScriptType;
use contract_core::data_parser::config_cell;
use contract_core::error::{AsI8, CoreError};
use contract_core::{cc_assert, util, verifiers, warn};
use types::constants::owner_lock;
use types::constants::Action::{DeployConfig, UpdateConfig};
use types::packed::Script;

use super::error::ConfigError;

pub fn main() -> Result<(), Box<dyn AsI8>> {
    debug!("====== Running config-cell-type ======");

    let action = util::get_tx_action()?;
    let self_script = Script::from(high_level::load_script().map_err(ConfigError::from)?);

    debug!("==== Action {} ====", action.to_string());

    verify_script_args_is_empty(&self_script)?;

    let (input_config_cells, output_config_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, self_script.as_reader())?;
    match action {
        DeployConfig => {
            verifiers::permission::verify_input_has_owner_lock(0)?;

            verifiers::basic::verify_cell_number_and_position(
                "ConfigCell",
                &input_config_cells,
                &[],
                &output_config_cells,
                &[0],
            )?;
        }
        UpdateConfig => {
            verifiers::basic::verify_cell_number_and_position(
                "ConfigCell",
                &input_config_cells,
                &[0],
                &output_config_cells,
                &[0],
            )?;
        }
        _ => {
            return Err(CoreError::ActionNotSupported {
                action: action.to_string(),
            }
            .into());
        }
    }

    verify_output_lock(output_config_cells[0])?;
    verify_output_data_format(output_config_cells[0])?;

    Ok(())
}

fn verify_script_args_is_empty(script: &Script) -> Result<(), Box<dyn AsI8>> {
    debug!("Verifying the args of current script is empty");

    cc_assert!(script.args().is_empty(), ConfigError::ArgsMustBeEmpty);

    Ok(())
}

fn verify_output_lock(index: usize) -> Result<(), Box<dyn AsI8>> {
    debug!("outputs[{}] Verifying the ConfigCell.lock is owner lock.", index);

    let lock = high_level::load_cell_lock(index, Source::Output).map_err(ConfigError::from)?;
    let owner_lock = owner_lock();

    cc_assert!(
        util::is_entity_eq(&lock, owner_lock),
        CoreError::CellLockMustBeOwnerLock {
            cell_name: String::from("ConfigCell")
        }
    );

    Ok(())
}

fn verify_output_data_format(index: usize) -> Result<(), Box<dyn AsI8>> {
    debug!(
        "outputs[{}] Verifying the ConfigCell.data has a valid data format.",
        index
    );

    let data = high_level::load_cell_data(index, Source::Output).map_err(ConfigError::from)?;

    match config_cell::parse(&data) {
        Ok((_version, _configs)) => {}
        Err(err) => {
            warn!("{}", err.to_string());
            return Err(err.into());
        }
    }

    Ok(())
}
