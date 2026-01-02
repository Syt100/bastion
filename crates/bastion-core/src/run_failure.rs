use std::fmt;

#[derive(Debug, Clone)]
pub struct RunFailedWithSummary {
    pub code: &'static str,
    pub summary: serde_json::Value,
    pub message: String,
}

impl RunFailedWithSummary {
    pub fn new(code: &'static str, message: impl Into<String>, summary: serde_json::Value) -> Self {
        Self {
            code,
            summary,
            message: message.into(),
        }
    }
}

impl fmt::Display for RunFailedWithSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RunFailedWithSummary {}
