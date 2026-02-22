use crate::{DbErrors, table_builder::TableBuilder};
use crate::custom_types::IdentityValue;
use std::path::PathBuf;
use std::fs::File;
use std::collections::HashMap;
use crate::PharaohDBState;
use std::io::SeekFrom;
use std::io::Seek;
use std::io::Read;
use std::io::Write;
use chrono::Utc;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng,PasswordHash, PasswordVerifier},
};
use serde_json::Value;
use crate::metadata::DbMetaData;
use std::fs::OpenOptions;
use uuid::Uuid;
use std::fs;
use crate::DBTypes;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
pub struct PharaohDatabase {
    pub name: String,
    pub size: usize,
    pub created_at: u64,
    pub secret_key: String,
    pub path: PathBuf,
    pub log_file: File,
    pub index: HashMap<IdentityValue, u64>,
    pub next_offset: u64,
    pub record_count: u64,
    pub sync_on_write: bool,
}


impl PharaohDatabase {
    pub fn create_db(name: String, secret_key: &str) -> Result<Self, DbErrors> {
    if name.is_empty() {
        return Err(DbErrors::Dbnamenotsupplied);
    };

    if secret_key.trim().is_empty() {
        return Err(DbErrors::Secretnotsupplied);
    }

    let folder = PathBuf::from(name.trim());

    fs::create_dir_all(&folder).map_err(|_| DbErrors::Cannotcreatefolder)?;

    let meta_dir = folder.join("META");
    let wal_dir = folder.join("WAL");
    let tables_dir = folder.join("TABLES");
    let indexes_dir = folder.join("INDEXES");

    fs::create_dir_all(&meta_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;
    fs::create_dir_all(&wal_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;
    fs::create_dir_all(&tables_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;
    fs::create_dir_all(&indexes_dir).map_err(|_| DbErrors::Cannotcreatefolder)?;

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
    let db_meta_bytes = fs::read(metadata_file).map_err(|_| DbErrors::Cannotreadmetadatafile)?;
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

    Ok(PharaohDatabase {
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
    
pub fn create_table(
     &mut self,
    builder: TableBuilder,
) -> Result<&mut Self, DbErrors> {
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
            .read(true)
            .append(true)
            .open(data_path)
            .map_err(|_| DbErrors::Cannotopenfile)?;

        let offset = file
            .seek(SeekFrom::End(0))
            .map_err(|_| DbErrors::Cannotdeserialize)?;
        file.write_all(&row_bytes)
            .map_err(|_| DbErrors::Cannotwritetofile)?;

        self.index.insert(id.to_string(), offset);
        self.next_offset += row_bytes.len() as u64;
        self.record_count += 1;

        Ok(id.clone())
    }

    pub fn find_where(&self, table_name: &str, field: &str, value: &Value) -> Vec<Value> {
        self.find_with_offset(table_name, field, value)
            .into_iter()
            .map(|(record, _)| record)
            .collect()
    }

    pub fn update_where(
        &mut self,
        table_name: &str,
        field: &str,
        value: &Value,
        new_data: Value,
    ) -> Result<u64, DbErrors> {
        let matches = self.find_with_offset(table_name, field, value);

        if matches.is_empty() {
            return Ok(0);
        }

        let data_path = self.path.join("TABLES").join(table_name).join("data.tbl");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&data_path)
            .map_err(|_| DbErrors::Cannotopenfile)?;

        let mut updated_count = 0;

        for (existing_record, old_offset) in matches {
            let id = existing_record
                .get("ID")
                .and_then(|v| v.as_str())
                .ok_or(DbErrors::Invalidinputformat)?
                .to_string();

            let mut merged = existing_record.clone();
            if let (Some(merged_obj), Some(new_obj)) =
                (merged.as_object_mut(), new_data.as_object())
            {
                for (k, v) in new_obj {
                    if k != "ID" {
                        merged_obj.insert(k.clone(), v.clone());
                    }
                }
            }

            let new_row_bytes = self.serialize_row(id.clone(), &merged)?;

            file.seek(SeekFrom::Start(old_offset))
                .map_err(|_| DbErrors::Cannotwritetofile)?;
            file.write_all(&[0x00])
                .map_err(|_| DbErrors::Cannotwritetofile)?;

            let new_offset = file
                .seek(SeekFrom::End(0))
                .map_err(|_| DbErrors::Cannotwritetofile)?;
            file.write_all(&new_row_bytes)
                .map_err(|_| DbErrors::Cannotwritetofile)?;

            self.index.insert(id, new_offset);
            updated_count += 1;
        }

        Ok(updated_count)
    }

    pub fn delete_db(&mut self) -> Result<(), DbErrors> {
        let db_path = PathBuf::from(self.name.trim());
        fs::remove_dir_all(db_path).map_err(|_| DbErrors::Cannotdeletedatabase)
    }

    fn next_identity(&self) -> IdentityValue {
        let new_row_id = Uuid::new_v4().to_string();

        new_row_id + Utc::now().to_string().as_str()
    }

    fn validate_insert(&self, schema: &TableBuilder, input: &Value) -> Result<(), DbErrors> {
        let obj = input.as_object().ok_or(DbErrors::Invalidinputformat)?;

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

    fn find_with_offset(&self, table_name: &str, field: &str, value: &Value) -> Vec<(Value, u64)> {
        let table_dir = self.path.join("TABLES").join(table_name);
        let data_path = table_dir.join("data.tbl");
        let mut results = vec![];
        let mut file = match File::open(&data_path) {
            Ok(f) => f,
            Err(_) => return results,
        };

        loop {
            let offset = match file.seek(SeekFrom::Current(0)) {
                Ok(o) => o,
                Err(_) => break,
            };

            let mut status = [0u8; 1];
            if file.read_exact(&mut status).is_err() {
                break;
            }

            let mut len_buf = [0u8; 4];
            if file.read_exact(&mut len_buf).is_err() {
                break;
            }

            let payload_len = u32::from_le_bytes(len_buf) as usize;

            let mut payload = vec![0u8; payload_len];
            if file.read_exact(&mut payload).is_err() {
                break;
            }

            if status[0] == 0x00 {
                continue;
            }

            let row_bytes: Vec<u8> = match wincode::deserialize(&payload) {
                Ok(b) => b,
                Err(_) => continue,
            };

            let row_value: Value = match serde_json::from_slice(&row_bytes) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if row_value.get(field) == Some(value) {
                results.push((row_value, offset));
            }
        }

        results
    }
    fn serialize_row(&self, id: IdentityValue, input: &Value) -> Result<Vec<u8>, DbErrors> {
        let mut row = HashMap::new();
        row.insert("ID".to_string(), Value::String(id));

        let obj = input.as_object().unwrap();
        for (k, v) in obj {
            row.insert(k.clone(), v.clone());
        }

        let row_bytes = serde_json::to_string(&row)
            .map_err(|_| DbErrors::Cannotserialize)?
            .into_bytes();

        let payload = wincode::serialize(&row_bytes).map_err(|_| DbErrors::Cannotserialize)?;

        let mut out = Vec::new();
        out.push(0x01);
        out.extend((payload.len() as u32).to_le_bytes());
        out.extend(payload);

        Ok(out)
    }

    pub fn delete_where(
        &mut self,
        table_name: &str,
        field: &str,
        value: &Value,
    ) -> Result<u64, DbErrors> {
        let matches = self.find_with_offset(table_name, field, value);

        if matches.is_empty() {
            return Ok(0);
        }

        let data_path = self.path.join("TABLES").join(table_name).join("data.tbl");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&data_path)
            .map_err(|_| DbErrors::Cannotopenfile)?;

        let mut deleted_count = 0;

        for (existing_record, old_offset) in matches {
            let id = existing_record
                .get("ID")
                .and_then(|v| v.as_str())
                .ok_or(DbErrors::Invalidinputformat)?
                .to_string();

            file.seek(SeekFrom::Start(old_offset))
                .map_err(|_| DbErrors::Cannotwritetofile)?;
            file.write_all(&[0x00])
                .map_err(|_| DbErrors::Cannotwritetofile)?;

            self.index.remove(&id);
            deleted_count += 1;
        }

        Ok(deleted_count)
    }

    pub fn find_all(&self, table_name: &str) -> Vec<Value> {
        let table_dir = self.path.join("TABLES").join(table_name);
        let data_path = table_dir.join("data.tbl");
        let mut results = vec![];

        let mut file = match File::open(&data_path) {
            Ok(f) => f,
            Err(_) => return results,
        };

        loop {
            let mut status = [0u8; 1];
            if file.read_exact(&mut status).is_err() {
                break;
            }

            let mut len_buf = [0u8; 4];
            if file.read_exact(&mut len_buf).is_err() {
                break;
            }

            let payload_len = u32::from_le_bytes(len_buf) as usize;

            let mut payload = vec![0u8; payload_len];
            if file.read_exact(&mut payload).is_err() {
                break;
            }

       
            if status[0] == 0x00 {
                continue;
            }

            let row_bytes: Vec<u8> = match wincode::deserialize(&payload) {
                Ok(b) => b,
                Err(_) => continue,
            };

            let row_value: Value = match serde_json::from_slice(&row_bytes) {
                Ok(v) => v,
                Err(_) => continue,
            };

        
            results.push(row_value);
        }

        results
    }

}