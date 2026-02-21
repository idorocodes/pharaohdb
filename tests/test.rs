#[cfg(test)]
mod tests {
    use super::*;
    use pharaohdb::*;
    use pretty_assertions::assert_eq;
    use std::{fs, thread::{self, sleep}, time};

    #[test]
    fn test_create_database_success() {
        let db_name = "test_db";
        let secret = "super_secret_key";

        let _ = fs::remove_dir_all(db_name);

        let db = PharaohDatabase::create(db_name.to_string(), secret)
            .expect("database creation should succeed");

        assert_eq!(db.name, db_name);
        assert_eq!(db.record_count, 0);
        assert_eq!(db.sync_on_write, true);

        assert!(db.path.exists());
        assert!(db.path.join("META/db.meta").exists());
        assert!(db.path.join("WAL/wal.log").exists());
        assert!(db.path.join("TABLES").exists());
        assert!(db.path.join("INDEXES").exists());

        fs::remove_dir_all(db_name).unwrap();
    }

    #[test]
    fn test_create_database_empty_name_fails() {
        let result = PharaohDatabase::create("".to_string(), "key");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_database_empty_secret_fails() {
        let result = PharaohDatabase::create("db".to_string(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_database_success() {
        let db_name = "test_open_db";
        let secret = "open_secret";

        let _ = fs::remove_dir_all(db_name);
        let _db = PharaohDatabase::create(db_name.to_string(), secret).unwrap();

        let db = PharaohDatabase::open(db_name, secret).unwrap();
        assert_eq!(db.name, db_name);
        assert_eq!(db.record_count, 0);
        assert_eq!(db.sync_on_write, true);
        assert!(db.path.exists());
        fs::remove_dir_all(db_name).unwrap();
    }

    #[test]
    fn test_open_database_wrong_secret_fails() {
        let db_name = "test_open_db_fail";
        let secret = "correct_secret";
        let _ = fs::remove_dir_all(db_name);
        let _db = PharaohDatabase::create(db_name.to_string(), secret).unwrap();

        let result = PharaohDatabase::open(db_name, "wrong_secret");
        assert!(result.is_err());
        fs::remove_dir_all(db_name).unwrap();
    }

    #[test]
    fn test_open_database_nonexistent_fails() {
        let db_name = "nonexistent_db";
        let _ = fs::remove_dir_all(db_name); // ensure it doesn't exist
        let result = PharaohDatabase::open(db_name, "any_secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_database_metadata_missing_fails() {
        let db_name = "test_meta_missing";
        let secret = "secret_meta";
        let _ = fs::remove_dir_all(db_name);
        let db = PharaohDatabase::create(db_name.to_string(), secret).unwrap();

        // remove metadata file to simulate missing metadata
        let meta_path = db.path.join("META").join("db.meta");
        fs::remove_file(meta_path).unwrap();

        let result = PharaohDatabase::open(db_name, secret);
        assert!(result.is_err());
        fs::remove_dir_all(db_name).unwrap();
    }

  
    fn setup_db() -> PharaohDatabase {
        let db_name = "test_table_db";
        let _ = fs::remove_dir_all(db_name);
        PharaohDatabase::create(db_name.to_string(), "secret").unwrap()
        
    }

    #[test]
    fn test_table_builder_new_has_id_field() {
        let table = TableBuilder::new("users");
        assert_eq!(table.fields.len(), 1);
        assert_eq!(table.fields[0].0, "ID");
        assert_eq!(table.fields[0].1, DBTypes::Identity);
    }

    #[test]
    fn test_add_fields_to_table_builder() {
        let mut table = TableBuilder::new("users");
        table
            .add_string_field("username", true)
            .add_integer_field("age", false)
            .add_boolean_field("active", false);

        assert_eq!(table.fields.len(), 4); // ID + 3 fields
        assert_eq!(table.fields[1].0, "username");
        assert_eq!(table.fields[2].0, "age");
        assert_eq!(table.fields[3].0, "active");
    }

    #[test]
    fn test_create_table_success() {
        let mut db = setup_db();

        let table = TableBuilder::new("users")
            .add_string_field("username", true)
            .add_integer_field("age", false)
            .build();

        db.create_table(table).unwrap();

        let table_dir = db.path.join("TABLES").join("users");
        assert!(table_dir.exists());
        assert!(table_dir.join("schema.tbl").exists());
        assert!(table_dir.join("data.tbl").exists());

      
        let meta_path = db.path.join("META").join("db.meta");
        let meta_bytes = fs::read(&meta_path).unwrap();
        let metadata: DbMetaData = wincode::deserialize(&meta_bytes).unwrap();
        assert!(metadata.schema_registry.contains_key("users"));
    }

    #[test]
    fn test_create_table_duplicate_name_fails() {
        let mut db = setup_db();

        let table = TableBuilder::new("users_duplicate").build();
        db.create_table(table).unwrap();

        let duplicate_table = TableBuilder::new("users_duplicate").build();
        let result = db.create_table(duplicate_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_table_invalid_identity_field_fails() {
        let mut db = setup_db();

       
        let mut table = TableBuilder::new("bad_table");
        table.add_primary_identity_field();

        let result = db.create_table(table);
        assert!(result.is_err());

        let time = time::Duration::from_millis(10000);
        thread::sleep(time);
         fs::remove_dir_all("test_table_db").unwrap();
    }
    
}

