use anyhow::{Error, Result};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
    time::SystemTime,
};
mod dbtypes;
use dbtypes::DBTypes;
mod error;
use error::DbErros;
struct TableBuilder {
    pub name: String,
    pub fields: Vec<(String, DBTypes, bool)>,
}

impl TableBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: vec![],
        }
    }

    pub fn add_string_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::PharaohString, is_unique));
        self
    }

    pub fn add_integer_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::PharaohInteger, is_unique));
        self
    }

    pub fn add_boolean_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::PharaohBoolean, is_unique));
        self
    }
    pub fn build(&self) -> Self {
        TableBuilder {
            name: self.name.clone(),
            fields: self.fields.clone(),
        }
    }
}

struct PharaohDatabase {
    name: String,
    size: usize,
    created_at: SystemTime,
    secret_key: String,
    path: PathBuf,
    log_file: File,
    index: HashMap<Vec<u8>, u64>,
    next_offset: u64,
    record_count: u64,
    sync_on_write: bool,
}

impl PharaohDatabase {
    pub fn create(&mut self, name: String, secret_key: &str) -> Result<bool, DbErros> {
        let sucess: bool;

        if name.is_empty() {
            return Err(DbErros::DBNAMENOTSUPPLIED);
        };

        if secret_key.is_empty() {
            return Err(DbErros::SECRETNOTSUPPLIED);
        }

        let folder_name = format!("./{}", name.trim());

        fs::create_dir(&folder_name).map_err(|_| DbErros::CANNOTCREATEFOLDER)?;

        self.path = PathBuf::from(folder_name.clone());

        self.name = name;
        self.created_at = SystemTime::now();
        self.secret_key = String::from(secret_key.trim());
        self.size = 0;

        let log_file = File::create_new("db.log").map_err(|_| DbErros::CANNOTCREATEFILE)?;

        self.log_file = log_file;
        self.index = HashMap::new();
        self.record_count = 0;
        self.sync_on_write = true;

        sucess = true;
        Ok(sucess)
    }
    pub fn open(&self, db_name: &str, secret_key: &str) -> Result<Self, Error> {
        todo!()
    }
    pub fn create_table(&mut self, builder: TableBuilder) -> Result<&mut Self, Error> {
        todo!()
    }
    pub fn table(self, table_name: &str) -> Result<&mut Self, Error> {
        todo!()
    }

    pub fn insert(&mut self, value: Value) -> Result<Self, Error> {
        todo!()
    }

    pub fn update(&mut self) -> Result<Self, Error> {
        todo!()
    }
    pub fn delete_all(&mut self) -> Result<Self, Error> {
        todo!()
    }

    pub fn delete(&mut self) -> Result<Self, Error> {
        todo!()
    }

    pub fn commit(&self) {
        todo!()
    }

    pub fn query(&self) -> Result<(), Error> {
        todo!()
    }

    pub fn query_all(&self) -> Result<(), Error> {
        todo!()
    }
}
