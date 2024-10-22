use compile::Compile;
use serde::{Deserialize, Serialize};

pub mod compile;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub compile: Compile,
}


