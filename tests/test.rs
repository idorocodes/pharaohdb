#[cfg(test)]
mod tests {
    use pharaohdb::*;
    use pretty_assertions::assert_eq;
    use serde_json::{json};

    use tempfile::TempDir;

   
    fn setup_db() -> (PharaohDatabase, TempDir) {
        let dir = TempDir::new().expect("Failed to create temporary directory");
        let db_name = dir.path().to_string_lossy().into_owned();

        let db = PharaohDatabase::create_db(db_name.clone(), "secret")
            .expect("Failed to create test database");

        (db, dir)
    }

    fn setup_db_with_users_table() -> (PharaohDatabase, TempDir) {
        let (mut db, dir) = setup_db();

        db.create_table(make_users_table())
            .expect("Failed to create users table");

        (db, dir)
    }

    fn make_users_table() -> TableBuilder {
        TableBuilder::new("users")
            .add_string_field("name", false)
            .add_string_field("email", true)
            .add_string_field("address", false)
            .build()
    }

    fn path_exists(db: &PharaohDatabase, rel: &str) -> bool {
        db.path.join(rel).exists()
    }

 
    #[test]
    fn test_create_database_success() {
        let (db, _temp) = setup_db();

        assert_eq!(db.name, db.path.to_string_lossy().as_ref());
        assert_eq!(db.record_count, 0);
        assert!(path_exists(&db, "META/db.meta"));
        assert!(path_exists(&db, "WAL/wal.log"));
        assert!(path_exists(&db, "TABLES"));
        assert!(path_exists(&db, "INDEXES"));
    }

    #[test]
    fn test_create_database_empty_name_fails() {
        let result = PharaohDatabase::create_db("".to_string(), "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_database_empty_secret_fails() {
        let dir = TempDir::new().unwrap();
        let name = dir.path().to_string_lossy().into_owned();

        let result = PharaohDatabase::create_db(name, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_database_whitespace_secret_fails() {
        let dir = TempDir::new().unwrap();
        let name = dir.path().to_string_lossy().into_owned();

        let result = PharaohDatabase::create_db(name, "   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_database_duplicates_moves_without_breaking() {
        let (db, _temp) = setup_db();
        let name = db.name.clone();

        let result = PharaohDatabase::create_db(name, "secret");
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_database_success() {
        let (db, _temp) = setup_db();
        let name = db.name.clone();

        let reopened = PharaohDatabase::open(&name, "secret").unwrap();
        assert_eq!(reopened.name, name);
    }

    #[test]
    fn test_open_database_wrong_secret_fails() {
        let (db, _temp) = setup_db();
        let name = db.name.clone();

        let result = PharaohDatabase::open(&name, "wrongsecret");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_database_nonexistent_fails() {
        let result = PharaohDatabase::open("this_db_does_not_exist_999", "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_database_missing_metadata_fails() {
        let (db, _temp) = setup_db();
        let name = db.name.clone();

        std::fs::remove_file(db.path.join("META/db.meta")).unwrap();
        let result = PharaohDatabase::open(&name, "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_database_missing_wal_fails() {
        let (db, _temp) = setup_db();
        let name = db.name.clone();

        std::fs::remove_file(db.path.join("WAL/wal.log")).unwrap();
        let result = PharaohDatabase::open(&name, "secret");
        assert!(result.is_err());
    }

    // ── Table builder ────────────────────────────────────────────────────────

    #[test]
    fn test_table_builder_starts_with_id() {
        let t = TableBuilder::new("users");
        assert_eq!(t.fields.len(), 1);
        assert_eq!(t.fields[0].0, "ID");
        assert_eq!(t.fields[0].1, DBTypes::Identity);
        assert!(t.fields[0].2); // unique
    }

    #[test]
    fn test_table_builder_add_fields() {
        let mut t = TableBuilder::new("products");
        t.add_string_field("name", true)
            .add_integer_field("stock", false)
            .add_boolean_field("active", false);

        assert_eq!(t.fields.len(), 4);
        assert_eq!(t.fields[1].0, "name");
        assert_eq!(t.fields[2].0, "stock");
        assert_eq!(t.fields[3].0, "active");
    }

    
    #[test]
    fn test_create_table_success() {
        let (mut db, _temp) = setup_db();
        db.create_table(make_users_table()).unwrap();

        assert!(path_exists(&db, "TABLES/users/schema.tbl"));
        assert!(path_exists(&db, "TABLES/users/data.tbl"));

        let meta_bytes = std::fs::read(db.path.join("META/db.meta")).unwrap();
        let meta: DbMetaData = wincode::deserialize(&meta_bytes).unwrap();
        assert!(meta.schema_registry.contains_key("users"));
    }

    #[test]
    fn test_create_table_duplicate_fails() {
        let (mut db, _temp) = setup_db();
        db.create_table(make_users_table()).unwrap();

        let result = db.create_table(make_users_table());
        assert!(result.is_err());
    }

    #[test]
    fn test_create_table_empty_name_fails() {
        let (mut db, _temp) = setup_db();
        let t = TableBuilder::new("").build();
        assert!(db.create_table(t).is_err());
    }

    #[test]
    fn test_create_table_duplicate_field_name_fails() {
        let (mut db, _temp) = setup_db();
        let mut t = TableBuilder::new("bad");
        t.add_string_field("email", false)
            .add_string_field("email", false);

        assert!(db.create_table(t).is_err());
    }

    #[test]
    fn test_create_table_two_identity_fields_fails() {
        let (mut db, _temp) = setup_db();
        let mut t = TableBuilder::new("bad");
        t.add_primary_identity_field(); // second ID

        assert!(db.create_table(t).is_err());
    }

    
    #[test]
    fn test_insert_success_returns_id() {
        let (mut db, _temp) = setup_db_with_users_table();

        let id = db
            .insert(
                "users",
                json!({
                    "name": "Alice",
                    "email": "alice@example.com",
                    "address": "123 Main St"
                }),
            )
            .unwrap();

        assert!(!id.is_empty());
        assert_eq!(db.record_count, 1);
    }

    #[test]
    fn test_insert_missing_field_fails() {
        let (mut db, _temp) = setup_db_with_users_table();

        let result = db.insert(
            "users",
            json!({
                "name": "Bob",
                "email": "bob@example.com"
            }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_into_nonexistent_table_fails() {
        let (mut db, _temp) = setup_db();

        let result = db.insert("ghost", json!({"name": "Nobody"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_multiple_records_increments_count() {
        let (mut db, _temp) = setup_db_with_users_table();

        for i in 0..5 {
            db.insert(
                "users",
                json!({
                    "name": format!("User {}", i),
                    "email": format!("user{}@example.com", i),
                    "address": "Somewhere"
                }),
            )
            .unwrap();
        }

        assert_eq!(db.record_count, 5);
    }

    #[test]
    fn test_insert_each_record_gets_unique_id() {
        let (mut db, _temp) = setup_db_with_users_table();

        let id1 = db
            .insert(
                "users",
                json!({"name": "Alice", "email": "a@a.com", "address": "A"}),
            )
            .unwrap();

        let id2 = db
            .insert(
                "users",
                json!({"name": "Bob", "email": "b@b.com", "address": "B"}),
            )
            .unwrap();

        assert_ne!(id1, id2);
    }


    #[test]
    fn test_find_where_returns_matching_record() {
        let (mut db, _temp) = setup_db_with_users_table();
        db.insert(
            "users",
            json!({"name": "Alice", "email": "alice@example.com", "address": "123 St"}),
        )
        .unwrap();

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["name"], "Alice");
    }

    #[test]
    fn test_find_where_no_match_returns_empty() {
        let (mut db, _temp) = setup_db_with_users_table();

        let results = db.find_where("users", "email", &json!("nobody@example.com"));
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_where_returns_multiple_matches() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "Same St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Bob", "email": "b@example.com", "address": "Same St"}),
        )
        .unwrap();

        let results = db.find_where("users", "address", &json!("Same St"));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_where_result_contains_id_field() {
        let (mut db, _temp) = setup_db_with_users_table();
        db.insert(
            "users",
            json!({"name": "Alice", "email": "alice@example.com", "address": "123"}),
        )
        .unwrap();

        let results = db.find_where("users", "name", &json!("Alice"));
        assert!(results[0].get("ID").is_some());
    }

    #[test]
    fn test_update_where_changes_field() {
        let (mut db, _temp) = setup_db_with_users_table();
        db.insert(
            "users",
            json!({"name": "Alice", "email": "alice@example.com", "address": "Old St"}),
        )
        .unwrap();

        let count = db
            .update_where(
                "users",
                "email",
                &json!("alice@example.com"),
                json!({"address": "New St"}),
            )
            .unwrap();

        assert_eq!(count, 1);

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["address"], "New St");
    }

    #[test]
    fn test_update_where_preserves_unchanged_fields() {
        let (mut db, _temp) = setup_db_with_users_table();
        db.insert(
            "users",
            json!({"name": "Alice", "email": "alice@example.com", "address": "Old St"}),
        )
        .unwrap();

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({"address": "New St"}),
        )
        .unwrap();

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(results[0]["name"], "Alice");
        assert_eq!(results[0]["email"], "alice@example.com");
    }

    #[test]
    fn test_update_where_does_not_change_id() {
        let (mut db, _temp) = setup_db_with_users_table();
        let id = db
            .insert(
                "users",
                json!({"name": "Alice", "email": "alice@example.com", "address": "St"}),
            )
            .unwrap();

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({"ID": "fake-id", "address": "New St"}),
        )
        .unwrap();

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(results[0]["ID"].as_str().unwrap(), id);
    }

    #[test]
    fn test_update_where_no_match_returns_zero() {
        let (mut db, _temp) = setup_db_with_users_table();

        let count = db
            .update_where(
                "users",
                "email",
                &json!("ghost@example.com"),
                json!({"name": "Ghost"}),
            )
            .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_update_where_old_record_no_longer_visible() {
        let (mut db, _temp) = setup_db_with_users_table();
        db.insert(
            "users",
            json!({"name": "Alice", "email": "alice@example.com", "address": "Old St"}),
        )
        .unwrap();

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({"address": "New St"}),
        )
        .unwrap();

        assert!(db.find_where("users", "address", &json!("Old St")).is_empty());
        assert_eq!(db.find_where("users", "address", &json!("New St")).len(), 1);
    }

    #[test]
    fn test_update_where_updates_multiple_matches() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "Same St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Bob", "email": "b@example.com", "address": "Same St"}),
        )
        .unwrap();

        let count = db
            .update_where(
                "users",
                "address",
                &json!("Same St"),
                json!({"address": "New St"}),
            )
            .unwrap();

        assert_eq!(count, 2);
        assert_eq!(db.find_where("users", "address", &json!("New St")).len(), 2);
    }


    #[test]
    fn test_delete_db_removes_folder() {
        let (mut db, _temp) = setup_db();
        let path = db.path.clone();

        db.delete_db().unwrap();

        assert!(!path.exists());
       
    }
    
    #[test]
    fn test_index_populated_after_insert() {
        let (mut db, _temp) = setup_db_with_users_table();

        let id = db
            .insert(
                "users",
                json!({"name": "Alice", "email": "alice@example.com", "address": "St"}),
            )
            .unwrap();

        assert!(db.index.contains_key(&id));
    }

    #[test]
    fn test_index_updated_after_update_where() {
        let (mut db, _temp) = setup_db_with_users_table();

        let id = db
            .insert(
                "users",
                json!({"name": "Alice", "email": "alice@example.com", "address": "Old"}),
            )
            .unwrap();

        let old_offset = *db.index.get(&id).unwrap();

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({"address": "New"}),
        )
        .unwrap();

        let new_offset = *db.index.get(&id).unwrap();
        assert_ne!(old_offset, new_offset);
    }

   
    #[test]
    fn test_full_insert_find_update_find_flow() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "alice@example.com", "address": "123 Old Street"}),
        )
        .unwrap();

        let before = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(before[0]["address"], "123 Old Street");

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({"address": "456 New Street"}),
        )
        .unwrap();

        let after = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(after.len(), 1);
        assert_eq!(after[0]["address"], "456 New Street");
        assert_eq!(after[0]["name"], "Alice");
    }

    #[test]
    fn test_multiple_tables_dont_interfere() {
        let (mut db, _temp) = setup_db();

        db.create_table(make_users_table()).unwrap();

        let products = TableBuilder::new("products")
            .add_string_field("name", true)
            .add_string_field("price", false)
            .build();

        db.create_table(products).unwrap();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "alice@example.com", "address": "St"}),
        )
        .unwrap();

        db.insert(
            "products",
            json!({"name": "Widget", "price": "9.99"}),
        )
        .unwrap();

        let users = db.find_where("users", "name", &json!("Alice"));
        let products_found = db.find_where("products", "name", &json!("Widget"));

        assert_eq!(users.len(), 1);
        assert_eq!(products_found.len(), 1);
    }

    // ── find_all tests ───────────────────────────────────────────────────────

    #[test]
    fn test_find_all_returns_all_records() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "A St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Bob", "email": "b@example.com", "address": "B St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Charlie", "email": "c@example.com", "address": "C St"}),
        )
        .unwrap();

        let all = db.find_all("users");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_find_all_empty_table_returns_empty() {
        let (mut db, _temp) = setup_db_with_users_table();
        let all = db.find_all("users");
        assert!(all.is_empty());
    }

    #[test]
    fn test_find_all_skips_tombstoned_records() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "A St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Bob", "email": "b@example.com", "address": "B St"}),
        )
        .unwrap();

        db.delete_where("users", "email", &json!("a@example.com")).unwrap();

        let all = db.find_all("users");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0]["name"], "Bob");
    }

    #[test]
    fn test_find_all_skips_records_updated_with_tombstone() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "Old St"}),
        )
        .unwrap();

        db.update_where(
            "users",
            "email",
            &json!("a@example.com"),
            json!({"address": "New St"}),
        )
        .unwrap();

        let all = db.find_all("users");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0]["address"], "New St");
    }

    #[test]
    fn test_find_all_records_contain_id_field() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "St"}),
        )
        .unwrap();

        let all = db.find_all("users");
        assert!(all[0].get("ID").is_some());
        assert!(!all[0]["ID"].as_str().unwrap().is_empty());
    }

  
    #[test]
    fn test_delete_where_removes_record() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "St"}),
        )
        .unwrap();

        let count = db
            .delete_where("users", "email", &json!("a@example.com"))
            .unwrap();
        assert_eq!(count, 1);

        assert!(db.find_where("users", "email", &json!("a@example.com")).is_empty());
    }

    #[test]
    fn test_delete_where_returns_correct_count() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "Same St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Bob", "email": "b@example.com", "address": "Same St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Charlie", "email": "c@example.com", "address": "Same St"}),
        )
        .unwrap();

        let count = db
            .delete_where("users", "address", &json!("Same St"))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_delete_where_no_match_returns_zero() {
        let (mut db, _temp) = setup_db_with_users_table();

        let count = db
            .delete_where("users", "email", &json!("ghost@example.com"))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_delete_where_record_not_visible_in_find_all() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Bob", "email": "b@example.com", "address": "St"}),
        )
        .unwrap();

        db.delete_where("users", "email", &json!("a@example.com")).unwrap();

        let all = db.find_all("users");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0]["name"], "Bob");
    }

    #[test]
    fn test_delete_where_record_not_visible_in_find_where() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "St"}),
        )
        .unwrap();

        db.delete_where("users", "email", &json!("a@example.com")).unwrap();

        assert!(db.find_where("users", "name", &json!("Alice")).is_empty());
    }

    #[test]
    fn test_delete_where_removes_from_index() {
        let (mut db, _temp) = setup_db_with_users_table();

        let id = db
            .insert(
                "users",
                json!({"name": "Alice", "email": "a@example.com", "address": "St"}),
            )
            .unwrap();

        assert!(db.index.contains_key(&id));

        db.delete_where("users", "email", &json!("a@example.com")).unwrap();

        assert!(!db.index.contains_key(&id));
    }

    #[test]
    fn test_delete_where_only_deletes_matching_records() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "St"}),
        )
        .unwrap();
        db.insert(
            "users",
            json!({"name": "Bob", "email": "b@example.com", "address": "St"}),
        )
        .unwrap();

        db.delete_where("users", "email", &json!("a@example.com")).unwrap();

        assert_eq!(db.find_where("users", "name", &json!("Bob")).len(), 1);
        assert!(db.find_where("users", "name", &json!("Alice")).is_empty());
    }

    #[test]
    fn test_delete_where_then_insert_same_field_value_works() {
        let (mut db, _temp) = setup_db_with_users_table();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "Old St"}),
        )
        .unwrap();

        db.delete_where("users", "email", &json!("a@example.com")).unwrap();

        db.insert(
            "users",
            json!({"name": "Alice", "email": "a@example.com", "address": "New St"}),
        )
        .unwrap();

        let results = db.find_where("users", "email", &json!("a@example.com"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["address"], "New St");
    }
}
