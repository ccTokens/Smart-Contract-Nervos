use alloc::format;
use alloc::string::{String, ToString};

use types::packed::Tick;
use types::prelude::Entity;

use crate::error::CoreError;

pub fn parse_data(data: &[u8]) -> Result<(u8, Tick), CoreError> {
    cc_assert!(
        data.len() > 2,
        CoreError::ParseCellDataFailed {
            cell_name: String::from("TickCell"),
            msg: "The data is too short.".to_string(),
        }
    );

    let version = data[0];
    let tick: Tick;
    match version {
        0 => {
            tick = Tick::from_compatible_slice(&data[1..]).map_err(|_| CoreError::ParseCellDataFailed {
                cell_name: String::from("TickCell"),
                msg: format!("Parse slice to TickCell failed."),
            })?;
        }
        _ => {
            return Err(CoreError::ParseCellDataVersionFailed {
                version,
                cell_name: String::from("TickCell"),
            });
        }
    }

    Ok((version, tick))
}
