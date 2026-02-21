use serde::{Deserialize, Serialize};
use wincode::SchemaWrite;

#[derive(Serialize, Deserialize, SchemaWrite)]
pub enum PharaohDBState {
    Creating,
    Ready,
    Corrupt,
}
