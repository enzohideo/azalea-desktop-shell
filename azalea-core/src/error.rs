#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file")]
    Io(#[from] std::io::Error),
    #[error("Missing extension")]
    MissingExtension,
    #[error("Parsing error")]
    ParsingError(String),
}
