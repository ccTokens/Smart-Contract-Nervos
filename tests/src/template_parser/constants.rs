use lazy_static::lazy_static;
use regex::Regex;

pub const OMNI_FLAG_MULTISIG: u8 = 6;
pub const OMNI_FLAG_NO_MODE: u8 = 0;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Source {
    Input = 1,
    Output = 2,
    CellDep = 3,
}

lazy_static! {
    pub static ref RE_VARIABLE: Regex = Regex::new(r"\{\{([\w\-\.]+)\}\}").unwrap();
}
