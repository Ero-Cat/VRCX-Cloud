pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database error: {message}")]
    Sqlite {
        message: String,
        category: Option<vrcx_0_persistence::SqliteErrorCategory>,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    PersistenceInvalidData(String),

    #[error("{0}")]
    RegistryPolicyInvalid(String),

    #[error("{0}")]
    WebClient(String),

    #[error("Update artifact is invalid: {0}")]
    UpdateArtifactInvalid(String),

    #[error("{0}")]
    Custom(String),

    #[error("{message}")]
    VrchatApi { status_code: i32, message: String },

    #[error("{0}")]
    AuthInteractionRequired(String),

    #[error("{reason}")]
    AuthSessionInvalidated {
        reason: String,
        status_code: Option<i32>,
    },
}

impl From<vrcx_0_persistence::Error> for Error {
    fn from(value: vrcx_0_persistence::Error) -> Self {
        match value {
            vrcx_0_persistence::Error::Database(message) => Error::Database(message),
            vrcx_0_persistence::Error::Sqlite { message, category } => {
                Error::Sqlite { message, category }
            }
            vrcx_0_persistence::Error::Io(error) => Error::Io(error),
            vrcx_0_persistence::Error::Json(error) => Error::Json(error),
            vrcx_0_persistence::Error::InvalidData(message) => {
                Error::PersistenceInvalidData(message)
            }
            vrcx_0_persistence::Error::Custom(message) => Error::Custom(message),
        }
    }
}

impl From<vrcx_0_media::Error> for Error {
    fn from(value: vrcx_0_media::Error) -> Self {
        match value {
            vrcx_0_media::Error::Io(error) => Error::Io(error),
            vrcx_0_media::Error::Custom(message) => Error::Custom(message),
        }
    }
}

impl From<vrcx_0_platform::Error> for Error {
    fn from(value: vrcx_0_platform::Error) -> Self {
        match value {
            vrcx_0_platform::Error::Io(error) => Error::Io(error),
            vrcx_0_platform::Error::Json(error) => Error::Json(error),
            vrcx_0_platform::Error::RegistryPolicyInvalid(message) => {
                Error::RegistryPolicyInvalid(message)
            }
            vrcx_0_platform::Error::Custom(message) => Error::Custom(message),
        }
    }
}

impl From<vrcx_0_application_core::Error> for Error {
    fn from(value: vrcx_0_application_core::Error) -> Self {
        match value {
            vrcx_0_application_core::Error::Database(message) => Error::Database(message),
            vrcx_0_application_core::Error::Sqlite { message, category } => {
                Error::Sqlite { message, category }
            }
            vrcx_0_application_core::Error::Io(error) => Error::Io(error),
            vrcx_0_application_core::Error::Json(error) => Error::Json(error),
            vrcx_0_application_core::Error::PersistenceInvalidData(message) => {
                Error::PersistenceInvalidData(message)
            }
            vrcx_0_application_core::Error::RegistryPolicyInvalid(message) => {
                Error::RegistryPolicyInvalid(message)
            }
            vrcx_0_application_core::Error::WebClient(message) => Error::WebClient(message),
            vrcx_0_application_core::Error::UpdateArtifactInvalid(message) => {
                Error::UpdateArtifactInvalid(message)
            }
            vrcx_0_application_core::Error::VrchatApi {
                status_code,
                message,
            } => Error::VrchatApi {
                status_code,
                message,
            },
            vrcx_0_application_core::Error::Custom(message) => Error::Custom(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_application_diagnostic_categories() {
        assert!(matches!(
            Error::from(vrcx_0_application_core::Error::PersistenceInvalidData(
                "invalid snapshot".into()
            )),
            Error::PersistenceInvalidData(message) if message == "invalid snapshot"
        ));
        assert!(matches!(
            Error::from(vrcx_0_application_core::Error::RegistryPolicyInvalid(
                "invalid registry".into()
            )),
            Error::RegistryPolicyInvalid(message) if message == "invalid registry"
        ));
        assert!(matches!(
            Error::from(vrcx_0_application_core::Error::WebClient(
                "request failed".into()
            )),
            Error::WebClient(message) if message == "request failed"
        ));
        assert!(matches!(
            Error::from(vrcx_0_application_core::Error::UpdateArtifactInvalid(
                "signature mismatch".into()
            )),
            Error::UpdateArtifactInvalid(message) if message == "signature mismatch"
        ));
    }
}
