use std::collections::HashMap;
use std::error::Error as StdError;

use log::debug;
use serde_json::Value;

use super::constants::RE_VARIABLE;

#[derive(Clone, Debug, Default)]
pub struct VarParser {
    var_map: HashMap<String, String>,
}

impl VarParser {
    pub fn new() -> VarParser {
        VarParser {
            var_map: HashMap::new(),
        }
    }

    pub fn register_var(&mut self, key: String, value: String) {
        self.var_map.insert(key, value);
    }

    pub fn parse(&self, field: &Value) -> Result<Option<String>, Box<dyn StdError>> {
        let ret = match field.as_str() {
            Some(field_str) => {
                // debug!("script parser: field_str: {}", field_str);
                // {
                //     for i in self.var_map.iter() {
                //         debug!("script parser: var_map: {} {}", i.0, i.1);
                //     }
                // }
                // If args is not empty, try to find and replace variables in args.
                if let Some(caps) = RE_VARIABLE.captures(field_str) {
                    // TODO Support multiple variables in one field.
                    let key = caps.get(1).map(|m| m.as_str()).unwrap();
                    let value = match self.var_map.get(key) {
                        Some(value) => value,
                        _ => return Err(format!("Undefined variable: {}", key).into()),
                    };

                    let ret = field_str.to_string().replace(&format!("{{{{{}}}}}", key), value);

                    debug!("Replace {} with {}", key, value);

                    Some(ret)
                } else {
                    None
                }
            }
            None => None,
        };

        Ok(ret)
    }
}
