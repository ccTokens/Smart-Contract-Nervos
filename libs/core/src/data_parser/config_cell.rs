use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};

use types::constants::ConfigKey;
use types::packed::BytesVec;
use types::prelude::Entity;
use types::util::hex_string;

use crate::error::CoreError;

pub fn parse(data: &[u8]) -> Result<(u8, Vec<(ConfigKey, Vec<u8>)>), CoreError> {
    cc_assert!(
        data.len() > 2,
        CoreError::ParseCellDataFailed {
            cell_name: String::from("ConfigCell"),
            msg: "The data is too short.".to_string(),
        }
    );

    let version = data[0];
    let mut configs = vec![];
    match version {
        0 => {
            let config_mol =
                BytesVec::from_compatible_slice(&data[1..]).map_err(|_| CoreError::ParseCellDataFailed {
                    cell_name: String::from("ConfigCell"),
                    msg: "Parse slice to BytesVec failed.".to_string(),
                })?;
            for (i, item) in config_mol.into_iter().enumerate() {
                let bytes = item.as_reader().raw_data();

                // Parse the key
                let key_bytes: [u8; 4] = (&bytes[0..4]).try_into().map_err(|_| CoreError::ParseCellDataFailed {
                    cell_name: String::from("ConfigCell"),
                    msg: format!("[{}] Parse [0..4] to [u8; 4] failed.", i),
                })?;

                let key = match ConfigKey::try_from(u32::from_le_bytes(key_bytes)) {
                    Ok(key) => key,
                    Err(_) => {
                        warn!(
                            "[{}] Parse [0..4]({}) to config key failed, the key is removed or not defined.",
                            i,
                            hex_string(key_bytes.as_ref())
                        );
                        continue;
                    }
                };

                // Parse the value
                let value = bytes[4..].to_vec();

                configs.push((key, value));
            }
        }
        _ => {
            return Err(CoreError::ParseCellDataVersionFailed {
                version,
                cell_name: String::from("ConfigCell"),
            });
        }
    }

    Ok((version, configs))
}
