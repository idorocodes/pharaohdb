#[cfg(test)]
mod tests {
    use pharaohdb::*;
    use std::fs;
    use pretty_assertions::{assert_eq};
    

    #[test]
    fn test_create_database_success() {
        let db_name = "test_pharaoh_db";
        let secret = "super_secret_key";

        let _ = fs::remove_dir_all(db_name);

        let db = PharaohDatabase::create(db_name.to_string(), secret)
            .expect("database creation should succeed");

        assert_eq!(db.name, db_name);
        assert_eq!(db.record_count, 0);
        assert_eq!(db.sync_on_write, true);

        assert_eq!(db.path.exists(),true);
        assert_eq!(db.path.join("META/db.meta").exists(),true);
        assert_eq!(db.path.join("WAL/wal.log").exists(),true);
        assert_eq!(db.path.join("TABLES").exists(),true);
        assert_eq!(db.path.join("INDEXES").exists(),true);

        fs::remove_dir_all(db_name).unwrap();
    }

    #[test]
    fn test_create_database_empty_name_fails() {
        let result = PharaohDatabase::create("".to_string(), "key");
        assert_eq!(result.is_err(),true);
    }

    #[test]
    fn test_create_database_empty_secret_fails() {
        let result = PharaohDatabase::create("db".to_string(), "");
        assert_eq!(result.is_err(),true);
    }
}
