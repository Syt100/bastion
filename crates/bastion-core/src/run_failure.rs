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

#[cfg(test)]
mod tests {
    use super::RunFailedWithSummary;

    #[test]
    fn display_uses_message_and_anyhow_downcast_works() {
        let rf = RunFailedWithSummary::new("code", "boom", serde_json::json!({ "k": "v" }));
        assert_eq!(rf.to_string(), "boom");

        let err = anyhow::Error::new(rf.clone());
        assert_eq!(err.to_string(), "boom");

        let down = err
            .downcast_ref::<RunFailedWithSummary>()
            .expect("downcast");
        assert_eq!(down.code, "code");
        assert_eq!(down.message, "boom");
        assert_eq!(down.summary, serde_json::json!({ "k": "v" }));
    }
}
