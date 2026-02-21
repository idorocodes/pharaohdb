use serde::{Deserialize, Serialize};
use wincode::SchemaWrite;

#[derive(Clone, Serialize, Deserialize, SchemaWrite)]
pub enum DBTypes {
    String,
    Boolean,
    Integer,
}
