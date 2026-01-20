
# Pharaoh DB

**Pharaoh DB** is a lightweight, embedded, file-based database management system written in Rust and encrypted with Egyptian Hieroglyph.  
It is designed for **high performance**, **simplicity**, and **structured data storage** in applications that require local, secure, and efficient persistence without external dependencies.

## Key Features

- Structured tables with strongly-typed fields
- Built-in security via per-database secret key
- Automatic timestamped creation metadata
- Purely file-based storage (no external server required)
- Optional sync-on-write for strong durability guarantees
- Built-in append-only log + in-memory indexing
- Support for multiple tables per database
- Capable of handling millions of records efficiently

## Supported Field Types

| Type              | Rust type   | Description                  | Indexed by default |
|-------------------|-------------|------------------------------|--------------------|
| `PharaohString`   | `String`    | UTF-8 text                   | Yes (when unique)  |
| `PharaohInteger`  | `i64`       | 64-bit signed integer        | Yes (when unique)  |
| `PharaohBoolean`  | `bool`      | Boolean value                | No                 |

## Installation

```bash
git clone https://github.com/idorocodes/pharaoh-db.git
cd pharaoh-db
cargo build --release
```

## Quick Start

```rust
use pharaoh_db::{PharaohDatabase, TableBuilder};
use serde_json::json;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    
    let mut db = PharaohDatabase::create(
        "my_app_db",
        "super_secret_key_2025",
        "./data/my_app_db",
        true,   
    )?;

    /
    let users_table = TableBuilder::new("users")
        .add_string_field("username", true)   
        .add_integer_field("age", false)
        .add_boolean_field("is_active", false)
        .build();

    db.create_table(users_table)?;


    let new_user = json!({
        "username": "idorocodes",
        "age": 20,
        "is_active": true
    });

    let record_id = db.insert("users", new_user)?;

    println!("Inserted user with internal ID: {}", record_id);

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


Feedback, contributions, and use-cases are very welcome!

