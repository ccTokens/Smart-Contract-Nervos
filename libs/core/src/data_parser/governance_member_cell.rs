use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use types::constants::GovernanceMemberRole;
use types::packed::GovernanceMembers;
use types::prelude::Entity;

use crate::error::CoreError;

pub fn parse_type_args(args: &[u8]) -> Result<(GovernanceMemberRole, Vec<u8>), CoreError> {
    cc_assert!(
        args.len() > 2,
        CoreError::ParseCellDataFailed {
            cell_name: String::from("GovernanceMemberCell"),
            msg: "The cell.type.args is too short.".to_string(),
        }
    );

    let role = GovernanceMemberRole::try_from(args[0]).map_err(|_| CoreError::ParseCellDataFailed {
        cell_name: String::from("GovernanceMemberCell"),
        msg: format!("The role in cell.type.args is unkown value {}", args[0]),
    })?;
    let cell_id = (&args[1..]).to_vec();
    cc_assert!(
        cell_id.len() == 32,
        CoreError::ParseCellDataFailed {
            cell_name: String::from("GovernanceMemberCell"),
            msg: format!(
                "The cell ID in cell.type.args should be 32 bytes, but {} bytes found",
                cell_id.len()
            )
        }
    );

    Ok((role, cell_id))
}

pub fn parse_data(data: &[u8]) -> Result<(u8, GovernanceMembers), CoreError> {
    cc_assert!(
        data.len() > 2,
        CoreError::ParseCellDataFailed {
            cell_name: String::from("GovernanceMemberCell"),
            msg: "The data is too short.".to_string(),
        }
    );

    let version = data[0];
    let governance_members: GovernanceMembers;
    match version {
        0 => {
            governance_members =
                GovernanceMembers::from_compatible_slice(&data[1..]).map_err(|_| CoreError::ParseCellDataFailed {
                    cell_name: String::from("GovernanceMemberCell"),
                    msg: format!("Parse slice to GovernanceMembers failed."),
                })?;
        }
        _ => {
            return Err(CoreError::ParseCellDataVersionFailed {
                version,
                cell_name: String::from("GovernanceMemberCell"),
            });
        }
    }

    Ok((version, governance_members))
}
