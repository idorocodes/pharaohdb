use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

#[derive(Clone, Serialize, Deserialize, SchemaWrite,SchemaRead)]
pub enum DBTypes {
    String,
    Boolean,
    Integer,
}
