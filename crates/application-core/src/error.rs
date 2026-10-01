#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database error: {message}")]
    Sqlite {
        message: String,
        category: Option<vrcx_0_contracts::SqliteErrorCategory>,
    },

    #[error("IO error: {0}")]
    Io(std::io::Error),

    #[error("JSON error: {0}")]
    Json(serde_json::Error),

    #[error("Update artifact is invalid: {0}")]
    UpdateArtifactInvalid(String),

    #[error("{0}")]
    PersistenceInvalidData(String),

    #[error("{0}")]
    RegistryPolicyInvalid(String),

    #[error("{0}")]
    WebClient(String),

    #[error("{message}")]
    VrchatApi { status_code: i32, message: String },

    #[error("{0}")]
    Custom(String),
}

impl<T> From<T> for Error
where
    T: vrcx_0_contracts::ApplicationErrorSource,
{
    fn from(value: T) -> Self {
        use vrcx_0_contracts::ApplicationErrorPayload;

        match value.into_application_error() {
            ApplicationErrorPayload::Database(message) => Self::Database(message),
            ApplicationErrorPayload::Sqlite { message, category } => {
                Self::Sqlite { message, category }
            }
            ApplicationErrorPayload::Io(error) => Self::Io(error),
            ApplicationErrorPayload::Json(error) => Self::Json(error),
            ApplicationErrorPayload::PersistenceInvalidData(message) => {
                Self::PersistenceInvalidData(message)
            }
            ApplicationErrorPayload::WebClient(message) => Self::WebClient(message),
            ApplicationErrorPayload::VrchatApi {
                status_code,
                message,
            } => Self::VrchatApi {
                status_code,
                message,
            },
            ApplicationErrorPayload::Custom(message) => Self::Custom(message),
        }
    }
}
