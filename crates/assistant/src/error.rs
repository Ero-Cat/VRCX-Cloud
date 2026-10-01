#[derive(Debug, thiserror::Error)]
pub enum AssistantError {
    #[error("assistant is not configured")]
    NotConfigured,
    #[error("assistant endpoint was removed: {0}")]
    EndpointRemoved(String),
    #[error("invalid LLM endpoint: {0}")]
    InvalidEndpoint(String),
    #[error("assistant session not found")]
    SessionNotFound,
    #[error("assistant LLM error: {0}")]
    Llm(#[from] crate::ports::AssistantLlmError),
    #[error("assistant MCP error: {0}")]
    Mcp(#[from] vrcx_0_mcp::McpError),
    #[error("assistant persistence error: {0}")]
    Persistence(#[from] crate::ports::AssistantPortError),
    #[error("{0}")]
    Custom(String),
}
