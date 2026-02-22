
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
use serde_json::json;

fn main() -> Result<(), DbErrors> {
    let db_name = "test_db";
    let secret_key = "my&strong&key";

    
    let mut  db = PharaohDatabase::create_db(db_name.to_string(), secret_key)?;

  
    let users_table = TableBuilder::new("users")
        .add_string_field("name", false)
        .add_boolean_field("is_rust_dev", false)
        .add_string_field("email", true)
        .add_integer_field("age", false)
        .build();

   
    db.create_table(users_table)?;

    println!("Database and tables created successfully");


    let mut db = PharaohDatabase::open(db_name, secret_key)?;

   
    let user_id = db.insert(
        "users",
        json!({
            "name": "Idorocodes",
            "is_rust_dev": true,
            "email": "idoroyen33@gmail.com",
            "age": 17
        }),
    )?;

    println!("Inserted user with ID: {}", user_id);

    db.update_where("users", "email", 
    &json!("idoroyen33@gmail.com"),
     json!({"age":60}))?;

    let results = db.find_where(
        "users",
        "email",
        &json!("idoroyen33@gmail.com"),
    );

    println!("User age: {:?}", results[0].get("age"));

    let data  = db.find_all("users");
    println!("{:?}",data);

    
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

