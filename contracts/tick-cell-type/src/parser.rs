use alloc::boxed::Box;
use alloc::vec::Vec;

use contract_core::data_parser;
use contract_core::error::AsI8;
use types::constants::TickType;
use types::packed::Script;

use crate::error::TickError;

pub struct TickCellData {
    pub version: u8,
    pub type_: TickType,
    pub token_id: Vec<u8>,
    pub value: u128,
    pub merchant: Script,
    pub coin_type: Vec<u8>,
    pub tx_hash: Vec<u8>,
    pub receipt_addr: Vec<u8>,
}

pub fn parse_tick(data: &[u8]) -> Result<TickCellData, Box<dyn AsI8>> {
    let (version, tick) = data_parser::tick_cell::parse_data(data)?;

    let type_ = tick.tick_type().as_slice()[0];
    let type_ = match TickType::try_from(type_) {
        Ok(type_) => type_,
        Err(_) => return Err(Box::new(TickError::UnsupportedTickType(type_))),
    };

    let token_id = tick.token_id().raw_data().to_vec();
    let value = u128::from(tick.value());
    let merchant = tick.merchant();
    let coin_type = tick.coin_type().raw_data().to_vec();
    let tx_hash = tick.tx_hash().raw_data().to_vec();
    let receipt_addr = tick.receipt_addr().raw_data().to_vec();

    Ok(TickCellData {
        version,
        type_,
        token_id,
        merchant,
        coin_type,
        tx_hash,
        receipt_addr,
        value,
    })
}
