use crate::state::PharaohDBState;
use crate::table_builder::TableBuilder;
use serde::*;
use std::collections::HashMap;
use wincode::*;

#[derive(Deserialize, Serialize, SchemaWrite, SchemaRead)]
pub struct DbMetaData {
    pub name: String,
    pub db_id: String,
    pub time_stamp: u64,
    pub database_version: String,
    pub secret_key_fingerprint: String,
    pub state: PharaohDBState,
    pub schema_registry: HashMap<String, TableBuilder>,
}
