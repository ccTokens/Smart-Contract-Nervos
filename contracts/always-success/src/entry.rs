use core::result::Result;

use ckb_std::debug;

use crate::error::Error;

pub fn main() -> Result<(), Error> {
    debug!("always-success contract start running");
    Ok(())
}
