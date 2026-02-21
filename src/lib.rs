use rand::random;
use ::serde::{Deserialize, Serialize};
use anyhow::{Error, Result};
// use redb::Value;
// use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use wincode::{SchemaRead, SchemaWrite};
mod dbtypes;
use dbtypes::DBTypes;
mod state;
use state::*;
use uuid::*;
mod error;
use error::DbErrors;

#[derive(Deserialize, Serialize, SchemaWrite,SchemaRead)]
pub struct TableBuilder {
    pub name: String,
    pub fields: Vec<(String, DBTypes, bool)>,
}

#[derive(Deserialize, Serialize, SchemaWrite, SchemaRead)]
pub struct DbMetaData {
    name: String,
    db_id: String,
    time_stamp: u64,
    database_version: String,
    secret_key_fingerprint: String,
    state: PharaohDBState,
    schema_registry: HashMap<String, TableBuilder>,
}

impl TableBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: vec![],
        }
    }
    // pub fn add_primary_identity_field(&mut self, primary_identity_name:&str) -> &mut Self{
    //     let random= rand::random::<u64>().to_string();
    //     let uuid = uuid::Uuid::new_v4().to_string().as_str();

    //     let identity = random  + uuid;

    // }

    pub fn add_string_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::String, is_unique));
        self
    }

    pub fn add_integer_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::Integer, is_unique));
        self
    }

    pub fn add_boolean_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::Boolean, is_unique));
        self
    }
    pub fn build(&self) -> Self {
        TableBuilder {
            name: self.name.clone(),
            fields: self.fields.clone(),
        }
    }
}

pub struct PharaohDatabase {
    pub name: String,
    pub size: usize,
    pub created_at: u64,
    pub secret_key: String,
    pub path: PathBuf,
    pub log_file: File,
    pub index: HashMap<Vec<u8>, u64>,
    pub next_offset: u64,
    pub record_count: u64,
    pub sync_on_write: bool,
}

impl PharaohDatabase {
    pub fn create(name: String, secret_key: &str) -> Result<Self, DbErrors> {
        if name.is_empty() {
            return Err(DbErrors::Dbnamenotsupplied);
        };

        if secret_key.trim().is_empty() {
            return Err(DbErrors::Secretnotsupplied);
        }

        let folder = PathBuf::from(name.trim());

        fs::create_dir(&folder).map_err(|_| DbErrors::Cannotcreatefolder)?;

        let meta_dir = folder.join("META");
        let wal_dir = folder.join("WAL");
        let tables_dir = folder.join("TABLES");
        let indexes_dir = folder.join("INDEXES");

        fs::create_dir(&meta_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;
        fs::create_dir(&wal_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;
        fs::create_dir(&tables_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;
        fs::create_dir(&indexes_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;

        let fingerprint = {
            let bytes = secret_key.as_bytes();
            let hash: Vec<u8> = bytes.iter().map(|b| b.wrapping_mul(31)).collect();
            format!("{:x?}", hash)
        };

        let meta_path = meta_dir.join("db.meta");

        let mut metadata = DbMetaData {
            name: name.clone(),
            db_id: Uuid::new_v4().to_string(),
            time_stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| DbErrors::Cannotgettime)?
                .as_secs(),
            database_version: "0.0.1".to_string(),
            secret_key_fingerprint: fingerprint,
            state: PharaohDBState::Creating,
            schema_registry: HashMap::new(),
        };

        let meta_bytes = wincode::serialize(&metadata).map_err(|_| DbErrors::Cannotserialize)?;
        fs::write(&meta_path, meta_bytes).map_err(|_| DbErrors::Cannotwritetofile)?;

        let wal_path = wal_dir.join("wal.log");
        let wal_file = File::create_new(&wal_path).map_err(|_| DbErrors::Cannotcreatefile)?;

        metadata.state = PharaohDBState::Ready;

        let meta_bytes = wincode::serialize(&metadata).map_err(|_| DbErrors::Cannotserialize)?;

        fs::write(&meta_path, meta_bytes).map_err(|_| DbErrors::Cannotwritetofile)?;

        let new_pharaoh_database = PharaohDatabase {
            path: folder,
            name: name.clone(),
            created_at: metadata.time_stamp,
            secret_key: String::from(secret_key.trim()),
            size: 0,
            log_file: wal_file,
            index: HashMap::new(),
            record_count: 0,
            sync_on_write: true,
            next_offset: 0,
        };

        Ok(new_pharaoh_database)
    }
    pub fn open(db_name: &str, secret_key: &str) -> Result<Self, DbErrors> {
        if db_name.trim().is_empty() {
            return Err(DbErrors::Dbnamenotsupplied);
        }

        if secret_key.trim().is_empty() {
            return Err(DbErrors::Secretnotsupplied);
        }

        let fingerprint = {
            let bytes = secret_key.as_bytes();
            let hash: Vec<u8> = bytes.iter().map(|b| b.wrapping_mul(31)).collect();
            format!("{:x?}", hash)
        };

        let db = PathBuf::from(db_name);

        if !db.exists() {
            return Err(DbErrors::Databasedoesnotexist);
        }

        let metadata_file = db.join("META").join("db.meta");
        let wal_file = db.join("WAL").join("wal.log");

        if !metadata_file.exists() {
            return Err(DbErrors::Metadatafiledoesnotexist);
        }

        if !wal_file.exists() {
            return Err(DbErrors::Walfiledoesnotexist);
        }
        let db_meta_bytes =
            fs::read(metadata_file).map_err(|_| DbErrors::Cannotreadmetadatafile)?;
        let meta_file: DbMetaData =
            wincode::deserialize(&db_meta_bytes).map_err(|_| DbErrors::Cannotdeserialize)?;

        if meta_file.name != db_name.trim() {
            return Err(DbErrors::Nodbfound);
        }

        if meta_file.secret_key_fingerprint != fingerprint {
            return Err(DbErrors::Wrongsecret);
        }

        if meta_file.state != PharaohDBState::Ready {
            return Err(DbErrors::Databasenotready);
        }

        let wal = OpenOptions::new()
            .read(true)
            .append(true)
            .open(wal_file)
            .map_err(|_| DbErrors::Cannotopenfile)?;

        Ok(Self {
            name: meta_file.name,
            path: db,
            created_at: meta_file.time_stamp,
            secret_key: secret_key.trim().to_string(),
            size: 0,
            log_file: wal,
            index: HashMap::new(),
            record_count: 0,
            next_offset: 0,
            sync_on_write: true,
        })
    }
    pub fn create_table(&mut self, builder: TableBuilder) -> Result<&mut Self, Error> {
           
        
        todo!()
    }
    pub fn _table(self, _table_name: &str) -> Result<&mut Self, Error> {
        todo!()
    }

    pub fn _insert(&mut self) -> Result<Self, Error> {
        todo!()
    }

    pub fn _update(&mut self) -> Result<Self, Error> {
        todo!()
    }
    pub fn _delete_all(&mut self) -> Result<Self, Error> {
        todo!()
    }

    pub fn _delete(&mut self) -> Result<Self, Error> {
        todo!()
    }

    pub fn _commit(&self) {
        todo!()
    }

    pub fn _query(&self) -> Result<(), Error> {
        todo!()
    }

    pub fn _query_all(&self) -> Result<(), Error> {
        todo!()
    }
}
