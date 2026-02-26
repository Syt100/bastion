use bastion_core::error_envelope::{
    ErrorEnvelopeV1, ErrorOriginV1, ErrorRetriableV1, ErrorTransportV1, LocalizedTextRefV1,
};
use serde::Serialize;
use serde_json::{Map, Value};

pub const EVENT_ERROR_ENVELOPE_FIELD: &str = "error_envelope";

pub fn insert_error_envelope(fields: &mut Map<String, Value>, envelope: ErrorEnvelopeV1) {
    let Ok(value) = serde_json::to_value(envelope) else {
        return;
    };
    fields.insert(EVENT_ERROR_ENVELOPE_FIELD.to_string(), value);
}

pub fn text_ref(key: &'static str) -> LocalizedTextRefV1 {
    LocalizedTextRefV1::new(key)
}

pub fn text_ref_with_params(
    key: &'static str,
    params: impl IntoIterator<Item = (&'static str, Value)>,
) -> LocalizedTextRefV1 {
    let mut out = LocalizedTextRefV1::new(key);
    for (param_key, value) in params {
        out.params.insert(param_key.to_string(), value);
    }
    out
}

pub fn retriable(value: bool) -> ErrorRetriableV1 {
    ErrorRetriableV1::new(value)
}

pub fn retriable_with_reason(value: bool, reason: impl Into<String>) -> ErrorRetriableV1 {
    ErrorRetriableV1::new(value).with_reason(reason)
}

pub fn retriable_with_reason_retry_after(
    value: bool,
    reason: impl Into<String>,
    retry_after_sec: Option<u64>,
) -> ErrorRetriableV1 {
    let mut out = ErrorRetriableV1::new(value).with_reason(reason);
    if let Some(secs) = retry_after_sec {
        out = out.with_retry_after_sec(secs);
    }
    out
}

pub fn with_context_param(
    mut envelope: ErrorEnvelopeV1,
    key: &'static str,
    value: impl Serialize,
) -> ErrorEnvelopeV1 {
    let v = serde_json::to_value(value).unwrap_or(Value::Null);
    let mut context = match envelope.context.take() {
        Some(Value::Object(obj)) => obj,
        _ => Map::new(),
    };
    context.insert(key.to_string(), v);
    envelope.context = Some(Value::Object(context));
    envelope
}

pub fn origin(layer: &'static str, component: &'static str, op: &'static str) -> ErrorOriginV1 {
    ErrorOriginV1::new(layer, component, op)
}

pub fn transport(protocol: &'static str) -> ErrorTransportV1 {
    ErrorTransportV1::new(protocol)
}

pub fn envelope(
    code: impl Into<String>,
    kind: impl Into<String>,
    retriable: ErrorRetriableV1,
    hint_key: &'static str,
    message_key: &'static str,
    transport: ErrorTransportV1,
) -> ErrorEnvelopeV1 {
    ErrorEnvelopeV1::new(
        code,
        kind,
        retriable,
        text_ref(hint_key),
        text_ref(message_key),
        transport,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        EVENT_ERROR_ENVELOPE_FIELD, envelope, insert_error_envelope, origin, retriable_with_reason,
        transport, with_context_param,
    };

    #[test]
    fn insert_error_envelope_writes_field() {
        let mut fields = serde_json::Map::new();
        let envelope = with_context_param(
            envelope(
                "target.auth.invalid_credentials",
                "auth",
                retriable_with_reason(false, "auth"),
                "diagnostics.hint.target.auth_invalid",
                "diagnostics.message.target.auth_failed",
                transport("http").with_status_code(401),
            )
            .with_origin(origin("target", "webdav", "get"))
            .with_stage("download"),
            "attempt",
            1,
        );

        insert_error_envelope(&mut fields, envelope);
        assert!(fields.contains_key(EVENT_ERROR_ENVELOPE_FIELD));
        assert_eq!(
            fields[EVENT_ERROR_ENVELOPE_FIELD]["transport"]["status_code"],
            401
        );
        assert_eq!(fields[EVENT_ERROR_ENVELOPE_FIELD]["context"]["attempt"], 1);
    }
}
