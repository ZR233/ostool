use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UbootConfig {
    pub serial: String,
    pub baud_rate: i64,
    pub net: String,
    pub dtb_file: String,
}

impl UbootConfig {
    pub fn config_by_select() -> Self {
        


        todo!()
    }
}
