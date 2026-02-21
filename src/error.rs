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
    #[error("Cannot get time")]
    Cannotgettime,
}
