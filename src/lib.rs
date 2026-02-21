use ::serde::{Deserialize, Serialize};
use anyhow::{Error, Result};
// use redb::Value;
// use serde_json::Value;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use wincode::{SchemaRead, SchemaWrite};
pub mod dbtypes;
pub use dbtypes::DBTypes;
pub mod state;
use chrono::prelude::*;
pub use state::*;
use uuid::*;
pub mod error;
pub use error::DbErrors;

type IdentityValue = String;
#[derive(Deserialize, Serialize, SchemaWrite, SchemaRead)]
pub struct TableBuilder {
    pub name: String,
    pub fields: Vec<(String, DBTypes, bool)>,
}

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

impl TableBuilder {
    pub fn new(name: &str) -> Self {
        let mut primary_field = vec![];

        primary_field.push(("ID".to_string(), DBTypes::Identity, true));
        Self {
            name: name.to_string(),

            fields: primary_field,
        }
    }
    pub fn add_primary_identity_field(&mut self) -> &mut Self {
        let id_string = "ID";

        self.fields
            .push((id_string.to_string(), DBTypes::Identity, true));
        self
    }

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

        // let fingerprint = {
        //     let bytes = secret_key.as_bytes();
        //     let hash: Vec<u8> = bytes.iter().map(|b| b.wrapping_mul(31)).collect();
        //     format!("{:x?}", hash)
        // };

        let password = secret_key.as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password, &salt)
            .map_err(|_| DbErrors::Cannothashpasword)?
            .to_string();

        let meta_path = meta_dir.join("db.meta");

        let mut metadata = DbMetaData {
            name: name.clone(),
            db_id: Uuid::new_v4().to_string(),
            time_stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| DbErrors::Cannotgettime)?
                .as_secs(),
            database_version: "0.0.1".to_string(),
            secret_key_fingerprint: password_hash,
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

        // let fingerprint = {
        //     let bytes = secret_key.as_bytes();
        //     let hash: Vec<u8> = bytes.iter().map(|b| b.wrapping_mul(31)).collect();
        //     format!("{:x?}", hash)
        // };

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

        let parsed_hash = PasswordHash::new(&meta_file.secret_key_fingerprint)
            .map_err(|_| DbErrors::Cannotrederivepassword)?;

        if Argon2::default()
            .verify_password(secret_key.as_bytes(), &parsed_hash)
            .is_err()
        {
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
    pub fn create_table(&mut self, builder: TableBuilder) -> Result<&mut Self, DbErrors> {
        if builder.name.trim().is_empty() {
            return Err(DbErrors::Tablenamerequired);
        }

        if builder.fields.is_empty() {
            return Err(DbErrors::Atleastonefieldrequired);
        }

        let mut seen_fields = HashMap::new();
        let mut identity_count = 0;

        for (field_name, field_type, _) in &builder.fields {
            if seen_fields.contains_key(field_name) {
                return Err(DbErrors::Duplicatefieldname);
            }
            seen_fields.insert(field_name, true);

            if *field_type == DBTypes::Identity {
                identity_count += 1;
            }
        }

        if identity_count != 1 {
            return Err(DbErrors::Invalididentityfield);
        }

        let meta_path = self.path.join("META").join("db.meta");
        if !meta_path.exists() {
            return Err(DbErrors::Metadatafiledoesnotexist);
        }

        let meta_bytes = fs::read(&meta_path).map_err(|_| DbErrors::Cannotreadmetadatafile)?;
        let mut metadata: DbMetaData =
            wincode::deserialize(&meta_bytes).map_err(|_| DbErrors::Cannotdeserialize)?;

        if metadata.state != PharaohDBState::Ready {
            return Err(DbErrors::Databasenotready);
        }

        if metadata.schema_registry.contains_key(&builder.name) {
            return Err(DbErrors::Tablealreadyexists);
        }

        let table_dir = self.path.join("TABLES").join(builder.name.trim());

        if table_dir.exists() {
            return Err(DbErrors::Tablealreadyexists);
        }

        fs::create_dir(&table_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;

        let schema_path = table_dir.join("schema.tbl");
        let schema_bytes = wincode::serialize(&builder).map_err(|_| DbErrors::Cannotserialize)?;
        fs::write(&schema_path, schema_bytes).map_err(|_| DbErrors::Cannotwritetofile)?;

        let data_path = table_dir.join("data.tbl");
        File::create(&data_path).map_err(|_| DbErrors::Cannotcreatefile)?;

        metadata
            .schema_registry
            .insert(builder.name.clone(), builder);

        let updated_meta = wincode::serialize(&metadata).map_err(|_| DbErrors::Cannotserialize)?;
        fs::write(&meta_path, updated_meta).map_err(|_| DbErrors::Cannotwritetofile)?;

        Ok(self)
    }

    pub fn insert(
        &mut self,
        table_name: &str,
        table_input: Value,
    ) -> Result<IdentityValue, DbErrors> {
        if table_name.trim().is_empty() {
            return Err(DbErrors::Tablenamerequired);
        }

        let table_dir = self.path.join("TABLES").join(table_name.trim());

        if !table_dir.exists() {
            return Err(DbErrors::Tablenotfound);
        }

        let schema_path = table_dir.join("schema.tbl");

        let schema_bytes = fs::read(&schema_path).map_err(|_| DbErrors::Cannotreadfile)?;

        let schema_data: TableBuilder =
            wincode::deserialize(&schema_bytes).map_err(|_| DbErrors::Cannotdeserialize)?;

        self.validate_insert(&schema_data, &table_input)?;
        let id = self.next_identity();
        let row_bytes = self.serialize_row(id.clone(), &table_input)?;

        let data_path = table_dir.join("data.tbl");
        let mut file = OpenOptions::new()
            .append(true)
            .open(data_path)
            .map_err(|_| DbErrors::Cannotopenfile)?;

        file.write_all(&row_bytes)
            .map_err(|_| DbErrors::Cannotwritetofile)?;

        
        self.record_count += 1;

        Ok(id.clone())
    }

    pub fn _update(&mut self) -> Result<Self, Error> {
        todo!()
    }
    pub fn _delete_all(&mut self) -> Result<Self, Error> {
        todo!()
    }

    fn next_identity(&self) -> IdentityValue {
        let new_row_id = Uuid::new_v4().to_string();

        let full_id = new_row_id + Utc::now().to_string().as_str();

        full_id
    }

    fn validate_insert(
    &self,
    schema: &TableBuilder,
    input: &Value,
) -> Result<(), DbErrors> {
    let obj = input.as_object()
        .ok_or(DbErrors::Invalidinputformat)?;

    for (field_name, field_type, _) in &schema.fields {
        if field_type == &DBTypes::Identity {
            continue; 
        }

        if !obj.contains_key(field_name) {
            return Err(DbErrors::Missingfield(field_name.clone()));
        }
    }

    Ok(())
}
    
    fn serialize_row(
    &self,
    id: IdentityValue,
    input: &Value,
) -> Result<Vec<u8>, DbErrors> {
    let mut row = HashMap::new();
    row.insert("ID".to_string(), Value::String(id));

    let obj = input.as_object().unwrap();
    for (k, v) in obj {
        row.insert(k.clone(), v.clone());
    }

    let row_bytes  = serde_json::to_string(&row).map_err(|_|DbErrors::Cannotserialize)?.into_bytes();
    

    let payload = wincode::serialize(&row_bytes)
        .map_err(|_| DbErrors::Cannotserialize)?;

    let mut out = Vec::new();
    out.extend((payload.len() as u32).to_le_bytes());
    out.extend(payload);

    Ok(out)
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
