use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, SchemaWrite, SchemaRead)]
pub enum DBTypes {
    String,
    Boolean,
    Integer,
    Identity,
}
