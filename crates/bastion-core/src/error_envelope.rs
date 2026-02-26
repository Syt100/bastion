use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

fn map_is_empty(value: &Map<String, Value>) -> bool {
    value.is_empty()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalizedTextRefV1 {
    pub key: String,
    #[serde(default, skip_serializing_if = "map_is_empty")]
    pub params: Map<String, Value>,
}

impl LocalizedTextRefV1 {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            params: Map::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.params.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorRetriableV1 {
    pub value: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_sec: Option<u64>,
}

impl ErrorRetriableV1 {
    pub fn new(value: bool) -> Self {
        Self {
            value,
            reason: None,
            retry_after_sec: None,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn with_retry_after_sec(mut self, retry_after_sec: u64) -> Self {
        self.retry_after_sec = Some(retry_after_sec);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorOriginV1 {
    pub layer: String,
    pub component: String,
    pub op: String,
}

impl ErrorOriginV1 {
    pub fn new(
        layer: impl Into<String>,
        component: impl Into<String>,
        op: impl Into<String>,
    ) -> Self {
        Self {
            layer: layer.into(),
            component: component.into(),
            op: op.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorTransportV1 {
    pub protocol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disconnect_code: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_error_code: Option<i32>,
}

impl ErrorTransportV1 {
    pub fn new(protocol: impl Into<String>) -> Self {
        Self {
            protocol: protocol.into(),
            status_code: None,
            status_text: None,
            provider: None,
            provider_code: None,
            provider_request_id: None,
            disconnect_code: None,
            io_kind: None,
            os_error_code: None,
        }
    }

    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn with_status_text(mut self, status_text: impl Into<String>) -> Self {
        self.status_text = Some(status_text.into());
        self
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_provider_code(mut self, provider_code: impl Into<String>) -> Self {
        self.provider_code = Some(provider_code.into());
        self
    }

    pub fn with_provider_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.provider_request_id = Some(request_id.into());
        self
    }

    pub fn with_disconnect_code(mut self, disconnect_code: u32) -> Self {
        self.disconnect_code = Some(disconnect_code);
        self
    }

    pub fn with_io_kind(mut self, io_kind: impl Into<String>) -> Self {
        self.io_kind = Some(io_kind.into());
        self
    }

    pub fn with_os_error_code(mut self, os_error_code: i32) -> Self {
        self.os_error_code = Some(os_error_code);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorEnvelopeV1 {
    pub schema_version: String,
    pub code: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<ErrorOriginV1>,
    pub retriable: ErrorRetriableV1,
    pub hint: LocalizedTextRefV1,
    pub message: LocalizedTextRefV1,
    pub transport: ErrorTransportV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug: Option<Value>,
}

impl ErrorEnvelopeV1 {
    pub const SCHEMA_VERSION: &'static str = "1.0";

    pub fn new(
        code: impl Into<String>,
        kind: impl Into<String>,
        retriable: ErrorRetriableV1,
        hint: LocalizedTextRefV1,
        message: LocalizedTextRefV1,
        transport: ErrorTransportV1,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            code: code.into(),
            kind: kind.into(),
            stage: None,
            origin: None,
            retriable,
            hint,
            message,
            transport,
            context: None,
            debug: None,
        }
    }

    pub fn with_stage(mut self, stage: impl Into<String>) -> Self {
        self.stage = Some(stage.into());
        self
    }

    pub fn with_origin(mut self, origin: ErrorOriginV1) -> Self {
        self.origin = Some(origin);
        self
    }

    pub fn with_context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_debug(mut self, debug: Value) -> Self {
        self.debug = Some(debug);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorEnvelopeV1, ErrorRetriableV1, ErrorTransportV1, LocalizedTextRefV1};

    #[test]
    fn localized_text_ref_omits_empty_params() {
        let payload = LocalizedTextRefV1::new("diagnostics.hint.example");
        let json = serde_json::to_value(&payload).expect("serialize");
        assert!(json.get("params").is_none());
    }

    #[test]
    fn envelope_round_trip_keeps_protocol_specific_fields() {
        let envelope = ErrorEnvelopeV1::new(
            "target.rate_limited",
            "rate_limited",
            ErrorRetriableV1::new(true)
                .with_reason("rate_limited")
                .with_retry_after_sec(3),
            LocalizedTextRefV1::new("diagnostics.hint.rate_limited").with_param("seconds", 3),
            LocalizedTextRefV1::new("diagnostics.message.rate_limited"),
            ErrorTransportV1::new("http")
                .with_status_code(429)
                .with_status_text("Too Many Requests"),
        )
        .with_stage("upload");

        let json = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(json["schema_version"], ErrorEnvelopeV1::SCHEMA_VERSION);
        assert_eq!(json["transport"]["protocol"], "http");
        assert_eq!(json["transport"]["status_code"], 429);
        assert_eq!(json["retriable"]["retry_after_sec"], 3);

        let parsed: ErrorEnvelopeV1 = serde_json::from_value(json).expect("deserialize");
        assert_eq!(parsed.transport.status_code, Some(429));
        assert_eq!(parsed.retriable.reason.as_deref(), Some("rate_limited"));
        assert_eq!(parsed.stage.as_deref(), Some("upload"));
    }
}
