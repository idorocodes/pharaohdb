use thiserror::Error;



#[derive(Error,Debug)]
pub enum DbErros{
    #[error("Database name is not supplied.!")]
    DBNAMENOTSUPPLIED,
    #[error("Secret key is not supplied. !")]
    SECRETNOTSUPPLIED,
    #[error("Cannot create file!")]
    CANNOTCREATEFILE,
    #[error("Cannor create folder")]
    CANNOTCREATEFOLDER}
