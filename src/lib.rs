use anyhow::{Error, Result};
use serde_json::Value;

struct TableBuilder {
    pub name: String,
    pub fields: Vec<(String, String, bool)>,
}

impl TableBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: vec![],
        }
    }

    pub fn add_string_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), "string".to_string(), is_unique));
        self
    }

    pub fn add_integer_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), "integer".to_string(), is_unique));
        self
    }

    pub fn add_boolean_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), "boolean".to_string(), is_unique));
        self
    }
    pub fn build(&self) -> Self {
        TableBuilder {
            name: self.name.clone(),
            fields: self.fields.clone(),
        }
    }
}

struct Database {
    
}

impl Database {
    pub fn open(&self, db_name: &str, secret_key: &str) -> Result<Self, Error> {
        todo!()
    }
    pub fn create_table(&mut self,builder:TableBuilder) ->Result<&mut Self,Error>{
        todo!()
    }
    pub fn table(self,table_name: &str) -> Result<&mut Self, Error> {
        todo!()
    }

    pub fn insert(&mut self, value: Value) -> Result<Self, Error> {
        todo!()
    }
    
    pub fn update(&mut self) -> Result<Self, Error> {
        todo!()
    }
    pub fn delete_all(&mut self) -> Result<Self, Error> {
        todo!()
    }
    
    pub fn delete(&mut self) -> Result<Self, Error> {
        todo!()
    }
    
    pub fn commit(&self) {
        todo!()
    }

    pub fn query(&self) -> Result<(), Error> {
        todo!()
    }
    
    pub fn query_all(&self) -> Result<(), Error> {
        todo!()
    }
}
