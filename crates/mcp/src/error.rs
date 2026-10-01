#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("failed to generate MCP token")]
    TokenGeneration(#[from] getrandom::Error),
    #[error("MCP IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("MCP application error: {0}")]
    Application(#[from] vrcx_0_application_core::Error),
    #[error("{0}")]
    Custom(String),
}
