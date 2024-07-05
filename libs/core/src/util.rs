use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use core::str::FromStr;

use ckb_std::ckb_constants::{CellField, Source};
use ckb_std::ckb_types::packed;
use ckb_std::ckb_types::prelude::Reader;
use ckb_std::error::SysError;
use ckb_std::{high_level, syscalls};
use types::constants::{Action, GovernanceMemberRole, OMNI_FLAG_MULTISIG, OMNI_FLAG_NO_MODE};
use types::packed::{Byte32, Byte32Reader, ScriptReader};
use types::prelude::Entity;
use types::util::{blake2b_256, new_blake2b};

use crate::constants::{ScriptType, LV_HEADER_LENGTH};
use crate::data_parser::governance_member_cell;
use crate::error::CoreError;

pub fn get_tx_action() -> Result<Action, CoreError> {
    let index = find_input_size()?;
    let witness = match high_level::load_witness(index, Source::Input) {
        Ok(witness) => witness,
        Err(_) => {
            warn!("{}", CoreError::ActionNotFound { index }.to_string());
            return Err(CoreError::ActionNotFound { index });
        }
    };

    let version_byte = witness[0];
    cc_assert!(
        version_byte == 0,
        CoreError::ActionVersionUnknown {
            index,
            version: version_byte
        }
    );

    let action_bytes = &witness[1..];
    let action_str = String::from_utf8(action_bytes.to_vec()).map_err(|_| CoreError::ActionUndefined {
        index,
        hex: hex::encode(action_bytes),
    })?;
    let action = Action::from_str(&action_str).map_err(|_| CoreError::ActionUndefined {
        index,
        hex: hex::encode(action_bytes),
    })?;

    Ok(action)
}

pub fn find_input_size() -> Result<usize, CoreError> {
    let mut i = 1;
    loop {
        let mut buf = [0u8; 1];
        match syscalls::load_input(&mut buf, 0, i, Source::Input) {
            Ok(_) => {
                // continue counting ...
            }
            Err(SysError::LengthNotEnough(_)) => {
                // continue counting ...
            }
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        i += 1;
    }

    Ok(i)
}

pub fn is_entity_eq<A: Entity, B: Entity>(a: &A, b: &B) -> bool {
    a.as_slice() == b.as_slice()
}

pub fn is_reader_eq<'a, T: Reader<'a>>(a: T, b: T) -> bool {
    a.as_slice() == b.as_slice()
}

pub fn find_cells_by_type_id(
    script_type: ScriptType,
    type_id: Byte32Reader,
    source: Source,
) -> Result<Vec<usize>, CoreError> {
    let mut i = 0;
    let mut cell_indexes = Vec::new();
    loop {
        let offset = 16;
        // Here we use 33 byt es to store code_hash and hash_type together.
        let mut code_hash = [0u8; 33];
        let ret = match script_type {
            ScriptType::Lock => syscalls::load_cell_by_field(&mut code_hash, offset, i, source, CellField::Lock),
            ScriptType::Type => syscalls::load_cell_by_field(&mut code_hash, offset, i, source, CellField::Type),
        };

        match ret {
            Ok(_) => {
                // Since script.as_slice().len() must larger than the length of code_hash.
                unreachable!()
            }
            Err(SysError::LengthNotEnough(_)) => {
                // Build an array with specific code_hash and hash_type
                let mut type_id_with_hash_type = [0u8; 33];
                let (left, _) = type_id_with_hash_type.split_at_mut(32);
                left.copy_from_slice(type_id.raw_data());
                type_id_with_hash_type[32] = ScriptType::Type as u8;
                if code_hash == type_id_with_hash_type {
                    cell_indexes.push(i);
                }
            }
            Err(SysError::ItemMissing) if script_type == ScriptType::Type => {}
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        i += 1;
    }

    Ok(cell_indexes)
}

pub fn find_cells_by_type_id_bytes(
    script_type: ScriptType,
    type_id: Vec<u8>,
    source: Source,
) -> Result<Vec<usize>, CoreError> {
    let type_id_byte32 = Byte32::try_from(type_id).map_err(|_| CoreError::Encoding)?;
    find_cells_by_type_id(script_type, type_id_byte32.as_reader(), source)
}

pub fn find_cells_by_type_id_in_inputs_and_outputs(
    script_type: ScriptType,
    type_id: Byte32Reader,
) -> Result<(Vec<usize>, Vec<usize>), CoreError> {
    let input_cells = find_cells_by_type_id(script_type, type_id, Source::Input)?;
    let output_cells = find_cells_by_type_id(script_type, type_id, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn find_cells_by_type_id_and_filter<F: Fn(usize, Source) -> Result<bool, CoreError>>(
    script_type: ScriptType,
    type_id: Byte32Reader,
    source: Source,
    filter: F,
) -> Result<Vec<usize>, CoreError> {
    let cell_indexes = find_cells_by_type_id(script_type, type_id, source)?;
    let mut ret = Vec::new();
    for i in cell_indexes {
        if filter(i, source)? {
            ret.push(i);
        }
    }

    Ok(ret)
}

pub fn find_only_cell_by_type_id(
    cell_name: &str,
    script_type: ScriptType,
    type_id: Byte32Reader,
    source: Source,
) -> Result<usize, CoreError> {
    let cells = find_cells_by_type_id(script_type, type_id, source)?;

    cc_assert!(
        cells.len() == 1,
        CoreError::InvalidTransactionStructure {
            msg: format!(
                "Only one {} expected existing in this transaction, but found {:?} in {:?}.",
                cell_name,
                cells.len(),
                source
            )
        }
    );

    Ok(cells[0])
}

pub fn find_cells_by_script(
    script_type: ScriptType,
    script: ScriptReader,
    source: Source,
) -> Result<Vec<usize>, CoreError> {
    let mut i = 0;
    let mut cell_indexes = Vec::new();
    let expected_hash = blake2b_256(script.as_slice());
    loop {
        let ret = match script_type {
            ScriptType::Lock => high_level::load_cell_lock_hash(i, source).map(Some),
            _ => high_level::load_cell_type_hash(i, source),
        };

        match ret {
            Ok(Some(hash)) if hash == expected_hash => {
                cell_indexes.push(i);
            }
            Ok(_) => {}
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        i += 1;
    }

    Ok(cell_indexes)
}

pub fn find_cells_by_script_in_inputs_and_outputs(
    script_type: ScriptType,
    script: ScriptReader,
) -> Result<(Vec<usize>, Vec<usize>), CoreError> {
    let input_cells = find_cells_by_script(script_type, script, Source::Input)?;
    let output_cells = find_cells_by_script(script_type, script, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn find_cells_by_script_and_filter<F: Fn(usize, Source) -> Result<bool, CoreError>>(
    script_type: ScriptType,
    script: ScriptReader,
    source: Source,
    filter: F,
) -> Result<Vec<usize>, CoreError> {
    let cell_indexes = find_cells_by_script(script_type, script, source)?;
    let mut ret = Vec::new();
    for i in cell_indexes {
        if filter(i, source)? {
            ret.push(i);
        }
    }

    Ok(ret)
}

pub fn build_type_id(input: &packed::CellInput, output_index: u64) -> [u8; 32] {
    let mut blake2b = new_blake2b();
    blake2b.update(input.as_slice());
    blake2b.update(&output_index.to_le_bytes());
    //debug!("blake2b.update(input.as_slice()), {}", hex_string(input.as_slice()));
    //debug!("blake2b.update(&output_index.to_le_bytes()), {}", hex_string(output_index.to_le_bytes().as_slice()));
    let mut type_id = [0; 32];
    blake2b.finalize(&mut type_id);
    // let script = build_type_id_script(input, output_index);
    // let type_id = blake2b_256(script.as_slice());
    type_id
}

pub fn build_multisig_args(require_first_n: u8, threshold: u8, pubkey_hashes: Vec<Vec<u8>>) -> Vec<u8> {
    let mut bytes = vec![0, require_first_n, threshold, pubkey_hashes.len() as u8];

    for pubkey_hash in pubkey_hashes.iter() {
        bytes.extend(pubkey_hash);
    }

    // println!("bytes: {:?}", bytes);

    let hash = blake2b_256(bytes);
    (&hash[..20]).to_vec()
}

pub fn build_omni_lock_multisig_args(require_first_n: u8, threshold: u8, pubkey_hashes: Vec<Vec<u8>>) -> Vec<u8> {
    let lock_args = build_multisig_args(require_first_n, threshold, pubkey_hashes);

    let mut ret = vec![OMNI_FLAG_MULTISIG];
    ret.extend(lock_args);
    ret.extend(&[OMNI_FLAG_NO_MODE]);

    ret
}

pub fn load_governance_member_type_info(
    index: usize,
    source: Source,
) -> Result<(GovernanceMemberRole, Vec<u8>), CoreError> {
    debug!(
        "{:?}[{}] Load information from GovernanceMemberCell.type.args .",
        source, index
    );

    let type_script_opt = high_level::load_cell_type(index, source).map_err(CoreError::from)?;
    let type_args = match type_script_opt {
        Some(type_) => type_.as_reader().args().raw_data().to_vec(),
        None => {
            return Err(CoreError::InvalidTransactionStructure {
                msg: format!("{:?}[{}] The cell.type must not be empty.", source, index),
            }
            .into());
        }
    };
    let (role, cell_id) = governance_member_cell::parse_type_args(&type_args)?;

    Ok((role, cell_id))
}

pub fn parse_lv_field<'a>(field_name: &str, bytes: &'a [u8], start: usize) -> Result<(usize, &'a [u8]), CoreError> {
    // Every field is start with 4 bytes of uint32 as its length.
    let length = match bytes.get(start..(start + LV_HEADER_LENGTH)) {
        Some(bytes) => u32::from_le_bytes(bytes.try_into().expect("slice with incorrect length")) as usize,
        None => {
            warn!(
                "cannot parse lv structure's length for field {} at start {}",
                field_name, start
            );
            return Err(CoreError::ParseLvFailed {
                field_name: field_name.to_string(),
            });
        }
    };

    // Slice the field base on the start and length.
    let from = start + LV_HEADER_LENGTH;
    let to = from + length;
    let field_bytes = match bytes.get(from..to) {
        Some(bytes) => bytes,
        None => {
            warn!(
                "cannot parse lv structure's value for field {} , expect {} in {}..{}",
                field_name, length, from, to
            );
            return Err(CoreError::ParseLvFailed {
                field_name: field_name.to_string(),
            });
        }
    };

    let new_start = start + LV_HEADER_LENGTH + length;
    Ok((new_start, field_bytes))
}
