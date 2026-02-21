use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

#[derive(Serialize, PartialEq, Deserialize, SchemaWrite, SchemaRead)]
pub enum PharaohDBState {
    Creating,
    Ready,
    Corrupt,
}
