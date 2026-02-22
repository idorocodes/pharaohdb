use crate::dbtypes::DBTypes;
use serde::*;
use wincode::*;
#[derive(Deserialize, Serialize, SchemaWrite, SchemaRead)]
pub struct TableBuilder {
    pub name: String,
    pub fields: Vec<(String, DBTypes, bool)>,
}

impl TableBuilder {
    pub fn new(name: &str) -> Self {
        let mut primary_field = vec![];

        primary_field.push(("ID".to_string(), DBTypes::Identity, true));
        Self {
            name: name.to_string(),

            fields: primary_field,
        }
    }
    pub fn add_primary_identity_field(&mut self) -> &mut Self {
        let id_string = "ID";

        self.fields
            .push((id_string.to_string(), DBTypes::Identity, true));
        self
    }

    pub fn add_string_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::String, is_unique));
        self
    }

    pub fn add_integer_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::Integer, is_unique));
        self
    }

    pub fn add_boolean_field(&mut self, name: &str, is_unique: bool) -> &mut Self {
        self.fields
            .push((name.to_string(), DBTypes::Boolean, is_unique));
        self
    }
    pub fn build(&self) -> Self {
        TableBuilder {
            name: self.name.clone(),
            fields: self.fields.clone(),
        }
    }
}
