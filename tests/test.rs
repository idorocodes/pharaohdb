#[cfg(test)]
mod tests {
    use pharaohdb::*;
    use std::fs;

    #[test]
    fn test_create_database_success() {
        let db_name = "test_pharaoh_db";
        let secret = "super_secret_key";

        let _ = fs::remove_dir_all(db_name);

        let db = PharaohDatabase::create(db_name.to_string(), secret)
            .expect("database creation should succeed");

        assert_eq!(db.name, db_name);
        assert_eq!(db.record_count, 0);
        assert!(db.sync_on_write);

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
}
