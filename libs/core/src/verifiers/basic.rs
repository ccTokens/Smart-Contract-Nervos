use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use types::packed::Uint64;

use crate::constants::CellField;
use crate::error::*;
use crate::util;

pub fn verify_cell_dep_number(
    cell_name: &str,
    current_deps: &[usize],
    expected_deps_len: usize,
) -> Result<(), CoreError> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    cc_assert!(
        current_deps.len() == expected_deps_len,
        CoreError::InvalidTransactionStructure {
            msg: format!(
                "{}",
                match expected_deps_len {
                    0 => format!("There should be none {} in cell_deps.", cell_name),
                    1 => format!("There should be only one {} in cell_deps.", cell_name),
                    _ => format!("There should be {} {}s in cell_deps.", expected_deps_len, cell_name),
                }
            )
        }
    );

    Ok(())
}

pub fn verify_cell_number_range(
    cell_name: &str,
    current_inputs: &[usize],
    expected_inputs_range: (Ordering, usize),
    current_outputs: &[usize],
    expected_outputs_range: (Ordering, usize),
) -> Result<(), CoreError> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    cc_assert!(
        current_inputs.len().cmp(&expected_inputs_range.1) == expected_inputs_range.0,
        CoreError::InvalidTransactionStructure {
            msg: format!(
                "{}",
                match expected_inputs_range.0 {
                    Ordering::Less => format!(
                        "There should be less than {} {}s in inputs.",
                        expected_inputs_range.1, cell_name
                    ),
                    Ordering::Greater => format!(
                        "There should be more than {} {}s in inputs.",
                        expected_inputs_range.1, cell_name
                    ),
                    Ordering::Equal => format!(
                        "There should be exactly {} {}s in inputs.",
                        expected_inputs_range.1, cell_name
                    ),
                }
            )
        }
    );

    cc_assert!(
        current_outputs.len().cmp(&expected_outputs_range.1) == expected_outputs_range.0,
        CoreError::InvalidTransactionStructure {
            msg: format!(
                "{}",
                match expected_outputs_range.0 {
                    Ordering::Less => format!(
                        "There should be less than {} {}s in outputs.",
                        expected_outputs_range.1, cell_name
                    ),
                    Ordering::Greater => format!(
                        "There should be more than {} {}s in outputs.",
                        expected_outputs_range.1, cell_name
                    ),
                    Ordering::Equal => format!(
                        "There should be exactly {} {}s in outputs.",
                        expected_outputs_range.1, cell_name
                    ),
                }
            )
        }
    );

    Ok(())
}

pub fn verify_cell_number_and_position(
    cell_name: &str,
    current_inputs: &[usize],
    expected_inputs: &[usize],
    current_outputs: &[usize],
    expected_outputs: &[usize],
) -> Result<(), CoreError> {
    debug!("Verify if the number and position of {}s is correct.", cell_name);

    cc_assert!(
        current_inputs == expected_inputs,
        CoreError::InvalidTransactionStructure {
            msg: format!(
                "{}",
                match expected_inputs.len() {
                    0 => format!("There should be none {} in inputs.", cell_name),
                    1 => format!(
                        "There should be only one {} in inputs[{}]",
                        cell_name, &expected_inputs[0]
                    ),
                    _ => format!(
                        "There should be {} {}s in inputs{:?}",
                        expected_inputs.len(),
                        cell_name,
                        expected_inputs
                    ),
                }
            )
        }
    );

    cc_assert!(
        current_outputs == expected_outputs,
        CoreError::InvalidTransactionStructure {
            msg: format!(
                "{}",
                match expected_outputs.len() {
                    0 => format!("There should be none {} in outputs.", cell_name),
                    1 => format!(
                        "There should be only one {} in outputs[{}]",
                        cell_name, &expected_outputs[0]
                    ),
                    _ => format!(
                        "There should be {} {}s in outputs{:?}",
                        expected_outputs.len(),
                        cell_name,
                        expected_outputs
                    ),
                }
            )
        }
    );

    Ok(())
}

/// WARNING! The witness will not be compared.
pub fn verify_cell_consistent_with_exception(
    cell_name: &str,
    input_cell_index: usize,
    output_cell_index: usize,
    except_fields: Vec<CellField>,
) -> Result<(), CoreError> {
    let input_cell = high_level::load_cell(input_cell_index, Source::Input).map_err(CoreError::from)?;
    let output_cell = high_level::load_cell(output_cell_index, Source::Output).map_err(CoreError::from)?;

    if !except_fields.contains(&CellField::Capacity) {
        debug!("Verify if the capacity of the {} is consistent ...", cell_name);

        let input_capacity = u64::from(Uint64::from(input_cell.capacity()));
        let output_capacity = u64::from(Uint64::from(output_cell.capacity()));
        debug!("input_capacity: {}", input_capacity);
        debug!("output_capacity: {}", output_capacity);
        cc_assert!(
            input_capacity <= output_capacity,
            CoreError::CellCapacityMustBeConsistent {
                cell_name: cell_name.to_string()
            }
        );
    }

    if !except_fields.contains(&CellField::Lock) {
        debug!("Verify if the lock script of the {} is consistent ...", cell_name);

        let input_lock = input_cell.lock();
        let output_lock = output_cell.lock();

        cc_assert!(
            util::is_entity_eq(&input_lock, &output_lock),
            CoreError::CellLockMustBeConsistent {
                cell_name: cell_name.to_string()
            }
        );
    }

    if !except_fields.contains(&CellField::Type) {
        debug!("Verify if the type script of the {} is consistent ...", cell_name);

        let input_type = input_cell.type_();
        let output_type = output_cell.type_();

        cc_assert!(
            util::is_entity_eq(&input_type, &output_type),
            CoreError::CellLockMustBeConsistent {
                cell_name: cell_name.to_string()
            }
        );
    }

    if !except_fields.contains(&CellField::Data) {
        debug!("Verify if the data of the {} is consistent ...", cell_name);

        let input_data = high_level::load_cell_data(input_cell_index, Source::Input).map_err(CoreError::from)?;
        let output_data = high_level::load_cell_data(output_cell_index, Source::Output).map_err(CoreError::from)?;

        cc_assert!(
            input_data == output_data,
            CoreError::CellDataMustBeConsistent {
                cell_name: cell_name.to_string()
            }
        );
    }

    Ok(())
}
