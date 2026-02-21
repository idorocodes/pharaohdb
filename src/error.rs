use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbErrors {
    #[error("Database name is not supplied.!")]
    Dbnamenotsupplied,
    #[error("Secret key is not supplied. !")]
    Secretnotsupplied,
    #[error("Cannot create file!")]
    Cannotcreatefile,
    #[error("Cannot create folder")]
    Cannotcreatefolder,
    #[error("Cannot Write to file")]
    Cannotwritetofile,
    #[error("Cannot convert into bytes")]
    Cannotserialize,
    #[error("Cannot convert into metadata")]
    Cannotdeserialize,
    #[error("Cannot get time")]
    Cannotgettime,
    #[error("The Specified database does not exist")]
    Databasedoesnotexist,
    #[error("Meta data file cannot be found")]
    Metadatafiledoesnotexist,
    #[error("Wal file cannnot be found")]
    Walfiledoesnotexist,
    #[error("Unable to read the metadata file")]
    Cannotreadmetadatafile,
    #[error("No database with that name found")]
    Nodbfound,
    #[error("The secret key fingerprint is wrong")]
    Wrongsecret,
    #[error("The database is not ready")]
    Databasenotready,
    #[error("Unable to open file")]
    Cannotopenfile,
    #[error("Table name does not exist")]
    Tablenamedoesnotexist,
    #[error("All tables are required to have at lease one field")]
    Atleastonefieldrequired,
    #[error("Field name already specified")]
    Duplicatefieldname,
    #[error("Cannot update metadata")]
    Cannotupdatemetadata,
    #[error("This field type does not exist")]
    Invalididentityfield,
    #[error("This table already exists ")]
    Tablealreadyexists,
    #[error("Cannot hash the password")]
    Cannothashpasword,
    #[error("Cannot rederive password")]
    Cannotrederivepassword,
    #[error("Table name not supplied")]
    Tablenamerequired,
    #[error("Unable to read file")]
    Cannotreadfile,
    #[error("Table not found, please create")]
    Tablenotfound,
    #[error("Field name does not exist !")]
    Missingfield(String),
    #[error("Recheck input, there is a problem with it")]
    Invalidinputformat

}
