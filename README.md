
# Pharaoh DB

**Pharaoh DB** is a lightweight, embedded, file-based database management system written in Rust and encrypted with Egyptian Hieroglyph.  
It is designed for **high performance**, **simplicity**, and **structured data storage** in applications that require local, secure, and efficient persistence without external dependencies.

## Key Features

- Structured tables with strongly-typed fields
- Built-in security via per-database secret key
- Automatic timestamped creation metadata
- Purely file-based storage (no external server required)
- sync-on-write for strong durability guarantees
- Built-in append-only log + in-memory indexing
- Support for multiple tables per database

## Supported Field Types

| Type              | Rust type   | Description                  | Indexed by default |
|-------------------|-------------|------------------------------|--------------------|
| `String`   | `String`    | UTF-8 text                   | Yes (when unique)  |
| `Integer`  | `i64`       | 64-bit signed integer        | Yes (when unique)  |
| `Boolean`  | `bool`      | Boolean value                | No                 |

## Installation

```bash
git clone https://github.com/idorocodes/pharaoh-db.git
cd pharaoh-db
cargo build --release
```

## Quick Start

```rust
use pharaohdb::{DbErrors, PharaohDatabase, TableBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
struct User {
    name: String,
    is_rust_dev: bool,
    email: String,
    age: i32,
}

fn main() -> Result<(), DbErrors> {
    let db_name = "test_db";
    let secret_key = "my&strong&key";

    let mut db = PharaohDatabase::create_db(db_name.to_string(), secret_key)?;

    let users_table = TableBuilder::new("users")
        .add_string_field("name", false)
        .add_boolean_field("is_rust_dev", false)
        .add_string_field("email", true)
        .add_integer_field("age", false)
        .build();

    // Use a clean match to handle existing tables
    if let Err(e) = db.create_table(users_table) {
        if !matches!(e, DbErrors::Tablealreadyexists) {
            return Err(e);
        }
    }

    let my_user = User {
        name: "Idorocodes".into(),
        is_rust_dev: true,
        email: "idoroyen33@gmail.com".into(),
        age: 17,
    };

    let email_to_find = my_user.email.clone();
    let existing = db.find_where("users", "email", &json!(email_to_find));

    if existing.is_empty() {
        let user_id = db.insert("users", serde_json::to_value(my_user).unwrap())?;
        println!("Inserted new user with ID: {}", user_id);
    } else {
        println!("User already exists, skipping insertion.");
    }

    db.update_where("users", "email", &json!(email_to_find), json!({"age": 60}))?;

    let results = db.find_where("users", "email", &json!(email_to_find));

    let user: User = serde_json::from_value(results[0].clone()).expect("Failed to deserialize");

    println!("User data is: {:?}", user);

    db.delete_where(
        "users",
        "email",
        &serde_json::to_value("alice@rust.org").unwrap(),
    )?;

  
    let data = db.find_all("users");

   
    let all_users: Vec<User> =
        serde_json::from_value(json!(data)).expect("Database data didn't match User struct");

    println!("Total records in table: {}", all_users.len());
    println!("{:#?}", all_users);
    Ok(())
}
```

## Core Structures

### `PharaohDatabase`

```rust
pub struct PharaohDatabase {
    pub name:        String,                    // Database identifier
    pub size:        u64,                       // Current file size in bytes
    pub created_at:  std::time::SystemTime,     // Creation timestamp
    pub secret_key:  String,                    // Authentication key
    pub path:        std::path::PathBuf,        // Base directory
    pub log_file:    std::fs::File,             // Append-only operation log
    pub index:       std::collections::HashMap<Vec<u8>, u64>, // Primary index
    pub next_offset: u64,                       // Next available write position
    pub record_count: u64,                      // Total number of records
    pub sync_on_write: bool,                    // fsync after every write?
}
```

### `TableBuilder`

```rust
pub struct TableBuilder {
    name:   String,
    fields: Vec<(String, DBTypes, bool)>,   // (name, type, unique)
}
```
### `Commands`

- **create_db** - Creates the database 
- **open** - Opens the database 
- **create_table** - Creates a table
- **insert** - Inserts a new entry into the db 
- **find_where** - Find an entry based on a given value
- **find_all** - Return all the data in the db 
- **update_where** - Updates an existing data and replace it with a given value
- **delete_db** - Deletes the database 
- **delete_where** - Delete an existing data based on a given value


Feedback, contributions, and use-cases are very welcome!

