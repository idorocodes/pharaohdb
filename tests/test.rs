#[cfg(test)]
mod tests {
    use pharaohdb::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use serde_json::json;

  

    fn unique_db(name: &str) -> String {
        let n = format!("{}_{}", name, uuid::Uuid::new_v4().simple());
        let _ = fs::remove_dir_all(&n);
        n
    }

    fn make_db(name: &str) -> PharaohDatabase {
        PharaohDatabase::create_db(name.to_string(), "secret").unwrap()
        

    }
    fn make_users_table() -> TableBuilder {
        TableBuilder::new("users")
            .add_string_field("name", false)
            .add_string_field("email", true)
            .add_string_field("address", false)
            .build()
    }

    fn cleanup(name: &str) {
        let _ = fs::remove_dir_all(name);
    }

    #[test]
    fn test_create_database_success() {
        let name = unique_db("create_success");
        let db = make_db(&name);

        assert_eq!(db.name, name);
        assert_eq!(db.record_count, 0);
        assert!(db.path.join("META/db.meta").exists());
        assert!(db.path.join("WAL/wal.log").exists());
        assert!(db.path.join("TABLES").exists());
        assert!(db.path.join("INDEXES").exists());

        cleanup(&name);
    }

    #[test]
    fn test_create_database_empty_name_fails() {
        assert!(PharaohDatabase::create_db("".to_string(), "key").is_err());
    }

    #[test]
    fn test_create_database_empty_secret_fails() {
        assert!(PharaohDatabase::create_db("anyname".to_string(), "").is_err());
    }

    #[test]
    fn test_create_database_whitespace_secret_fails() {
        assert!(PharaohDatabase::create_db("anyname2".to_string(), "   ").is_err());
    }

    #[test]
    fn test_create_database_duplicate_fails() {
        let name = unique_db("create_dup");
        let _db = make_db(&name);
        // creating again over same folder should fail
        let result = PharaohDatabase::create_db(name.clone(), "secret");
        assert!(result.is_err());
        cleanup(&name);
    }

   
    #[test]
    fn test_open_database_success() {
        let name = unique_db("open_success");
        let _created = make_db(&name);
        let db = PharaohDatabase::open(&name, "secret").unwrap();
        assert_eq!(db.name, name);
        cleanup(&name);
    }

    #[test]
    fn test_open_database_wrong_secret_fails() {
        let name = unique_db("open_wrong_secret");
        let _db = make_db(&name);
        assert!(PharaohDatabase::open(&name, "wrong").is_err());
        cleanup(&name);
    }

    #[test]
    fn test_open_database_nonexistent_fails() {
        assert!(PharaohDatabase::open("does_not_exist_xyz", "secret").is_err());
    }

    #[test]
    fn test_open_database_missing_metadata_fails() {
        let name = unique_db("open_no_meta");
        let db = make_db(&name);
        fs::remove_file(db.path.join("META/db.meta")).unwrap();
        assert!(PharaohDatabase::open(&name, "secret").is_err());
        cleanup(&name);
    }

    #[test]
    fn test_open_database_missing_wal_fails() {
        let name = unique_db("open_no_wal");
        let db = make_db(&name);
        fs::remove_file(db.path.join("WAL/wal.log")).unwrap();
        assert!(PharaohDatabase::open(&name, "secret").is_err());
        cleanup(&name);
    }

    // ── table builder ─────────────────────────────────────────────────────────

    #[test]
    fn test_table_builder_starts_with_id() {
        let t = TableBuilder::new("users");
        assert_eq!(t.fields.len(), 1);
        assert_eq!(t.fields[0].0, "ID");
        assert_eq!(t.fields[0].1, DBTypes::Identity);
        assert_eq!(t.fields[0].2, true); // unique
    }

    #[test]
    fn test_table_builder_add_fields() {
        let mut t = TableBuilder::new("products");
        t.add_string_field("name", true)
            .add_integer_field("stock", false)
            .add_boolean_field("active", false);

        assert_eq!(t.fields.len(), 4); // ID + 3
        assert_eq!(t.fields[1].0, "name");
        assert_eq!(t.fields[2].0, "stock");
        assert_eq!(t.fields[3].0, "active");
    }

    // ── create table ──────────────────────────────────────────────────────────

    #[test]
    fn test_create_table_success() {
        let name = unique_db("tbl_success");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        assert!(db.path.join("TABLES/users/schema.tbl").exists());
        assert!(db.path.join("TABLES/users/data.tbl").exists());

        // check schema registry updated in metadata
        let meta_bytes = fs::read(db.path.join("META/db.meta")).unwrap();
        let meta: DbMetaData = wincode::deserialize(&meta_bytes).unwrap();
        assert!(meta.schema_registry.contains_key("users"));

        cleanup(&name);
    }

    #[test]
    fn test_create_table_duplicate_fails() {
        let name = unique_db("tbl_dup");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();
        let result = db.create_table(make_users_table());
        assert!(result.is_err());
        cleanup(&name);
    }

    #[test]
    fn test_create_table_empty_name_fails() {
        let name = unique_db("tbl_empty_name");
        let mut db = make_db(&name);
        let t = TableBuilder::new("").build();
        assert!(db.create_table(t).is_err());
        cleanup(&name);
    }

    #[test]
    fn test_create_table_duplicate_field_name_fails() {
        let name = unique_db("tbl_dup_field");
        let mut db = make_db(&name);
        let mut t = TableBuilder::new("bad");
        t.add_string_field("email", false)
            .add_string_field("email", false); // duplicate
        assert!(db.create_table(t).is_err());
        cleanup(&name);
    }

    #[test]
    fn test_create_table_two_identity_fields_fails() {
        let name = unique_db("tbl_two_ids");
        let mut db = make_db(&name);
        let mut t = TableBuilder::new("bad");
        t.add_primary_identity_field(); // adds a second ID on top of the one in new()
        assert!(db.create_table(t).is_err());
        cleanup(&name);
    }

    // ── insert ────────────────────────────────────────────────────────────────

    #[test]
    fn test_insert_success_returns_id() {
        let name = unique_db("insert_ok");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        let id = db.insert("users", json!({
            "name": "Alice",
            "email": "alice@example.com",
            "address": "123 Main St"
        })).unwrap();

        assert!(!id.is_empty());
        assert_eq!(db.record_count, 1);
        cleanup(&name);
    }

    #[test]
    fn test_insert_missing_field_fails() {
        let name = unique_db("insert_missing");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        // missing address field
        let result = db.insert("users", json!({
            "name": "Bob",
            "email": "bob@example.com"
        }));
        assert!(result.is_err());
        cleanup(&name);
    }

    #[test]
    fn test_insert_into_nonexistent_table_fails() {
        let name = unique_db("insert_no_table");
        let mut db = make_db(&name);
        let result = db.insert("ghost", json!({ "name": "Nobody" }));
        assert!(result.is_err());
        cleanup(&name);
    }

    #[test]
    fn test_insert_multiple_records_increments_count() {
        let name = unique_db("insert_multi");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        for i in 0..5 {
            db.insert("users", json!({
                "name": format!("User {}", i),
                "email": format!("user{}@example.com", i),
                "address": "Somewhere"
            })).unwrap();
        }

        assert_eq!(db.record_count, 5);
        cleanup(&name);
    }

    #[test]
    fn test_insert_each_record_gets_unique_id() {
        let name = unique_db("insert_unique_ids");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        let id1 = db.insert("users", json!({
            "name": "Alice", "email": "a@a.com", "address": "A"
        })).unwrap();

        let id2 = db.insert("users", json!({
            "name": "Bob", "email": "b@b.com", "address": "B"
        })).unwrap();

        assert_ne!(id1, id2);
        cleanup(&name);
    }

    // ── find_where ────────────────────────────────────────────────────────────

    #[test]
    fn test_find_where_returns_matching_record() {
        let name = unique_db("find_ok");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();
        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "123 St"
        })).unwrap();

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name").unwrap(), "Alice");
        cleanup(&name);
    }

    #[test]
    fn test_find_where_no_match_returns_empty() {
        let name = unique_db("find_empty");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        let results = db.find_where("users", "email", &json!("nobody@example.com"));
        assert!(results.is_empty());
        cleanup(&name);
    }

    #[test]
    fn test_find_where_returns_multiple_matches() {
        let name = unique_db("find_multi");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        // insert two users with same address (address is not unique)
        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "Same St"
        })).unwrap();
        db.insert("users", json!({
            "name": "Bob", "email": "bob@example.com", "address": "Same St"
        })).unwrap();

        let results = db.find_where("users", "address", &json!("Same St"));
        assert_eq!(results.len(), 2);
        cleanup(&name);
    }

    #[test]
    fn test_find_where_result_contains_id_field() {
        let name = unique_db("find_has_id");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();
        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "123"
        })).unwrap();

        let results = db.find_where("users", "name", &json!("Alice"));
        assert!(results[0].get("ID").is_some());
        cleanup(&name);
    }


    #[test]
    fn test_update_where_changes_field() {
        let name = unique_db("update_ok");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();
        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "Old St"
        })).unwrap();

        let count = db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({ "address": "New St" })
        ).unwrap();

        assert_eq!(count, 1);

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("address").unwrap(), "New St");
        cleanup(&name);
    }

    #[test]
    fn test_update_where_preserves_unchanged_fields() {
        let name = unique_db("update_preserves");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();
        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "Old St"
        })).unwrap();

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({ "address": "New St" }) // only updating address
        ).unwrap();

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        // name and email should be unchanged
        assert_eq!(results[0].get("name").unwrap(), "Alice");
        assert_eq!(results[0].get("email").unwrap(), "alice@example.com");
        cleanup(&name);
    }

    #[test]
    fn test_update_where_does_not_change_id() {
        let name = unique_db("update_id_safe");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();
        let id = db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "St"
        })).unwrap();

        // try to overwrite ID
        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({ "ID": "fake-id", "address": "New St" })
        ).unwrap();

        let results = db.find_where("users", "email", &json!("alice@example.com"));
        // ID must be the original, never overwritten
        assert_eq!(results[0].get("ID").unwrap().as_str().unwrap(), id);
        cleanup(&name);
    }

    #[test]
    fn test_update_where_no_match_returns_zero() {
        let name = unique_db("update_no_match");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        let count = db.update_where(
            "users",
            "email",
            &json!("ghost@example.com"),
            json!({ "name": "Ghost" })
        ).unwrap();

        assert_eq!(count, 0);
        cleanup(&name);
    }

    #[test]
    fn test_update_where_old_record_no_longer_visible() {
        let name = unique_db("update_tombstone");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();
        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "Old St"
        })).unwrap();

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({ "address": "New St" })
        ).unwrap();

        // searching by old address should return nothing
        let old_results = db.find_where("users", "address", &json!("Old St"));
        assert!(old_results.is_empty());

        // searching by new address should return one result
        let new_results = db.find_where("users", "address", &json!("New St"));
        assert_eq!(new_results.len(), 1);
        cleanup(&name);
    }

    #[test]
    fn test_update_where_updates_multiple_matches() {
        let name = unique_db("update_multi");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "Same St"
        })).unwrap();
        db.insert("users", json!({
            "name": "Bob", "email": "bob@example.com", "address": "Same St"
        })).unwrap();

        let count = db.update_where(
            "users",
            "address",
            &json!("Same St"),
            json!({ "address": "New St" })
        ).unwrap();

        assert_eq!(count, 2);

        let new_results = db.find_where("users", "address", &json!("New St"));
        assert_eq!(new_results.len(), 2);
        cleanup(&name);
    }

    // ── delete_db ─────────────────────────────────────────────────────────────

    #[test]
    fn test_delete_db_removes_folder() {
        let name = unique_db("delete_db");
        let mut db = make_db(&name);
        let path = db.path.clone();
        db.delete_db().unwrap();
        assert!(!path.exists());
    }

    // ── index integrity ───────────────────────────────────────────────────────

    #[test]
    fn test_index_populated_after_insert() {
        let name = unique_db("index_check");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        let id = db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "St"
        })).unwrap();

        assert!(db.index.contains_key(&id));
        cleanup(&name);
    }

    #[test]
    fn test_index_updated_after_update_where() {
        let name = unique_db("index_update");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        let id = db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "Old"
        })).unwrap();

        let old_offset = *db.index.get(&id).unwrap();

        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({ "address": "New" })
        ).unwrap();

        let new_offset = *db.index.get(&id).unwrap();

        // offset must have changed since record was appended at new position
        assert_ne!(old_offset, new_offset);
        cleanup(&name);
    }

    // ── end to end ────────────────────────────────────────────────────────────

    #[test]
    fn test_full_insert_find_update_find_flow() {
        let name = unique_db("e2e");
        let mut db = make_db(&name);
        db.create_table(make_users_table()).unwrap();

        // insert
        db.insert("users", json!({
            "name": "Alice",
            "email": "alice@example.com",
            "address": "123 Old Street"
        })).unwrap();

        // find and verify original
        let before = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(before[0].get("address").unwrap(), "123 Old Street");

        // update
        db.update_where(
            "users",
            "email",
            &json!("alice@example.com"),
            json!({ "address": "456 New Street" })
        ).unwrap();

        // find and verify updated
        let after = db.find_where("users", "email", &json!("alice@example.com"));
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].get("address").unwrap(), "456 New Street");
        assert_eq!(after[0].get("name").unwrap(), "Alice");

        cleanup(&name);
    }

    #[test]
    fn test_multiple_tables_dont_interfere() {
        let name = unique_db("multi_table");
        let mut db = make_db(&name);

        db.create_table(make_users_table()).unwrap();

        let products = TableBuilder::new("products")
            .add_string_field("name", true)
            .add_string_field("price", false)
            .build();
        db.create_table(products).unwrap();

        db.insert("users", json!({
            "name": "Alice", "email": "alice@example.com", "address": "St"
        })).unwrap();

        db.insert("products", json!({
            "name": "Widget", "price": "9.99"
        })).unwrap();

        let users = db.find_where("users", "name", &json!("Alice"));
        let products = db.find_where("products", "name", &json!("Widget"));

        assert_eq!(users.len(), 1);
        assert_eq!(products.len(), 1);

        cleanup(&name);
    }
    // Add these tests into your existing tests module

// ── find_all ─────────────────────────────────────────────────────────────────

#[test]
fn test_find_all_returns_all_records() {
    let name = unique_db("find_all_basic");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "A St"
    })).unwrap();
    db.insert("users", json!({
        "name": "Bob", "email": "bob@example.com", "address": "B St"
    })).unwrap();
    db.insert("users", json!({
        "name": "Charlie", "email": "charlie@example.com", "address": "C St"
    })).unwrap();

    let all = db.find_all("users");
    assert_eq!(all.len(), 3);

    cleanup(&name);
}

#[test]
fn test_find_all_empty_table_returns_empty() {
    let name = unique_db("find_all_empty");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    let all = db.find_all("users");
    assert!(all.is_empty());

    cleanup(&name);
}

#[test]
fn test_find_all_skips_tombstoned_records() {
    let name = unique_db("find_all_tombstone");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "A St"
    })).unwrap();
    db.insert("users", json!({
        "name": "Bob", "email": "bob@example.com", "address": "B St"
    })).unwrap();

    // delete Alice
    db.delete_where("users", "email", &json!("alice@example.com")).unwrap();

    // find_all should only return Bob
    let all = db.find_all("users");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].get("name").unwrap(), "Bob");

    cleanup(&name);
}

#[test]
fn test_find_all_skips_records_updated_with_tombstone() {
    let name = unique_db("find_all_update_tombstone");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "Old St"
    })).unwrap();

    // update creates a tombstone of the old record and appends a new one
    db.update_where(
        "users",
        "email",
        &json!("alice@example.com"),
        json!({ "address": "New St" })
    ).unwrap();

    // find_all should only see one Alice, not two
    let all = db.find_all("users");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].get("address").unwrap(), "New St");

    cleanup(&name);
}

#[test]
fn test_find_all_records_contain_id_field() {
    let name = unique_db("find_all_has_id");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "St"
    })).unwrap();

    let all = db.find_all("users");
    assert!(all[0].get("ID").is_some());
    assert!(!all[0].get("ID").unwrap().as_str().unwrap().is_empty());

    cleanup(&name);
}

// ── delete_where ──────────────────────────────────────────────────────────────

#[test]
fn test_delete_where_removes_record() {
    let name = unique_db("delete_removes");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "St"
    })).unwrap();

    let count = db.delete_where("users", "email", &json!("alice@example.com")).unwrap();
    assert_eq!(count, 1);

    // should no longer be findable
    let results = db.find_where("users", "email", &json!("alice@example.com"));
    assert!(results.is_empty());

    cleanup(&name);
}

#[test]
fn test_delete_where_returns_correct_count() {
    let name = unique_db("delete_count");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    // insert three users at the same address
    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "Same St"
    })).unwrap();
    db.insert("users", json!({
        "name": "Bob", "email": "bob@example.com", "address": "Same St"
    })).unwrap();
    db.insert("users", json!({
        "name": "Charlie", "email": "charlie@example.com", "address": "Same St"
    })).unwrap();

    // delete all at Same St
    let count = db.delete_where("users", "address", &json!("Same St")).unwrap();
    assert_eq!(count, 3);

    cleanup(&name);
}

#[test]
fn test_delete_where_no_match_returns_zero() {
    let name = unique_db("delete_no_match");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    let count = db.delete_where("users", "email", &json!("ghost@example.com")).unwrap();
    assert_eq!(count, 0);

    cleanup(&name);
}

#[test]
fn test_delete_where_record_not_visible_in_find_all() {
    let name = unique_db("delete_find_all");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "St"
    })).unwrap();
    db.insert("users", json!({
        "name": "Bob", "email": "bob@example.com", "address": "St"
    })).unwrap();

    db.delete_where("users", "email", &json!("alice@example.com")).unwrap();

    let all = db.find_all("users");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].get("name").unwrap(), "Bob");

    cleanup(&name);
}

#[test]
fn test_delete_where_record_not_visible_in_find_where() {
    let name = unique_db("delete_find_where");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "St"
    })).unwrap();

    db.delete_where("users", "email", &json!("alice@example.com")).unwrap();

    let results = db.find_where("users", "name", &json!("Alice"));
    assert!(results.is_empty());

    cleanup(&name);
}

#[test]
fn test_delete_where_removes_from_index() {
    let name = unique_db("delete_index");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    let id = db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "St"
    })).unwrap();

    // confirm in index before delete
    assert!(db.index.contains_key(&id));

    db.delete_where("users", "email", &json!("alice@example.com")).unwrap();

    // should be gone from index after delete
    assert!(!db.index.contains_key(&id));

    cleanup(&name);
}

#[test]
fn test_delete_where_only_deletes_matching_records() {
    let name = unique_db("delete_selective");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "St"
    })).unwrap();
    db.insert("users", json!({
        "name": "Bob", "email": "bob@example.com", "address": "St"
    })).unwrap();

    // only delete Alice
    db.delete_where("users", "email", &json!("alice@example.com")).unwrap();

    // Bob should still exist
    let results = db.find_where("users", "name", &json!("Bob"));
    assert_eq!(results.len(), 1);

    // Alice should be gone
    let results = db.find_where("users", "name", &json!("Alice"));
    assert!(results.is_empty());

    cleanup(&name);
}

#[test]
fn test_delete_where_then_insert_same_field_value_works() {
    let name = unique_db("delete_reinsert");
    let mut db = make_db(&name);
    db.create_table(make_users_table()).unwrap();

    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "Old St"
    })).unwrap();

    db.delete_where("users", "email", &json!("alice@example.com")).unwrap();

    // insert a new Alice with same email
    db.insert("users", json!({
        "name": "Alice", "email": "alice@example.com", "address": "New St"
    })).unwrap();

    let results = db.find_where("users", "email", &json!("alice@example.com"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].get("address").unwrap(), "New St");

    cleanup(&name);
}
}