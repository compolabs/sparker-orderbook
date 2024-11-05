use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),

    #[error("Parse: {0}")]
    Parse(#[from] ParseError),

    #[error("Json: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Database: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Tonic: {0}")]
    Tonic(#[from] tonic::transport::Error),

    #[error("Pangea: {0}")]
    PangeaClient(#[from] pangea_client::Error),

    #[error("Fuel: {0}")]
    Fuel(#[from] fuels::types::errors::Error),

    #[error("Invalid fuel chain id")]
    InvalidChainId,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("ParseInt: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Chrono: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("Hex: {0}")]
    FromHex(#[from] rustc_hex::FromHexError),

    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
}

macro_rules! impl_parse_error {
    ($($source:ty),*) => {
        $(
            impl From<$source> for Error {
                fn from(err: $source) -> Self {
                    Error::Parse(err.into())
                }
            }
        )*
    };
}

impl_parse_error!(
    std::num::ParseIntError,
    chrono::ParseError,
    rustc_hex::FromHexError,
    std::string::FromUtf8Error
);
