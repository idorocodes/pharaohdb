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
    #[error("Cannot convert metadata into bytes")]
    Cannotserialize,
    #[error("Cannot convert bytes into metadata")]
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
    #[error("TNo database with that name found")]
    Nodbfound,
    #[error("The secret key fingerprint is wrong")]
    Wrongsecret,
    #[error("The database is not ready")]
    Databasenotready,
    #[error("Unable to open file")]
    Cannotopenfile
}
