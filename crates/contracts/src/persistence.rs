#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqliteErrorCategory {
    Malformed,
    DiskFull,
    Locked,
    IoError,
}

#[derive(Debug)]
pub enum ApplicationErrorPayload {
    Database(String),
    Sqlite {
        message: String,
        category: Option<SqliteErrorCategory>,
    },
    Io(std::io::Error),
    Json(serde_json::Error),
    PersistenceInvalidData(String),
    WebClient(String),
    VrchatApi {
        status_code: i32,
        message: String,
    },
    Custom(String),
}

pub trait ApplicationErrorSource {
    fn into_application_error(self) -> ApplicationErrorPayload;
}

impl ApplicationErrorSource for std::io::Error {
    fn into_application_error(self) -> ApplicationErrorPayload {
        ApplicationErrorPayload::Io(self)
    }
}

impl ApplicationErrorSource for serde_json::Error {
    fn into_application_error(self) -> ApplicationErrorPayload {
        ApplicationErrorPayload::Json(self)
    }
}
