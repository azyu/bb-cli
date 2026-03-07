use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum CliError {
    InvalidInput(String),
    Config(String),
    NotLoggedIn,
    UnsupportedOutput(String),
    Api { status: u16, body: String },
    Io(String),
    Git(String),
    Internal(String),
}

impl CliError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid_input",
            Self::Config(_) => "config_error",
            Self::NotLoggedIn => "not_logged_in",
            Self::UnsupportedOutput(_) => "unsupported_output",
            Self::Api { .. } => "api_error",
            Self::Io(_) => "io_error",
            Self::Git(_) => "git_error",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidInput(message)
            | Self::Config(message)
            | Self::UnsupportedOutput(message)
            | Self::Io(message)
            | Self::Git(message)
            | Self::Internal(message) => message.clone(),
            Self::NotLoggedIn => "not logged in: run `bb auth login`".to_string(),
            Self::Api { status, body } => {
                if body.trim().is_empty() {
                    format!("api request failed: status {status}")
                } else {
                    format!("api request failed: status {status}: {}", body.trim())
                }
            }
        }
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Internal(error.to_string())
    }
}
