use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmtpTlsMode {
    None,
    Starttls,
    Implicit,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmtpSecretPayload {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: Vec<String>,
    pub tls: SmtpTlsMode,
}

pub async fn send_plain_text(
    payload: &SmtpSecretPayload,
    subject: &str,
    body: &str,
) -> Result<(), anyhow::Error> {
    let from: lettre::message::Mailbox = payload.from.parse()?;
    let mut builder = Message::builder()
        .from(from)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN);

    for to in &payload.to {
        let mailbox: lettre::message::Mailbox = to.parse()?;
        builder = builder.to(mailbox);
    }

    let email = builder.body(body.to_string())?;

    let mut mailer_builder = match payload.tls {
        SmtpTlsMode::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(payload.host.clone())
        }
        SmtpTlsMode::Starttls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&payload.host)?
        }
        SmtpTlsMode::Implicit => AsyncSmtpTransport::<Tokio1Executor>::relay(&payload.host)?,
    };

    mailer_builder = mailer_builder.port(payload.port);
    if !payload.username.trim().is_empty() {
        mailer_builder = mailer_builder.credentials(Credentials::new(
            payload.username.clone(),
            payload.password.clone(),
        ));
    }

    let mailer: AsyncSmtpTransport<Tokio1Executor> = mailer_builder.build();
    mailer.send(email).await?;
    Ok(())
}

pub fn is_valid_mailbox(addr: &str) -> bool {
    addr.parse::<lettre::message::Mailbox>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::is_valid_mailbox;

    #[test]
    fn mailbox_valid_simple_address() {
        assert!(is_valid_mailbox("user@example.com"));
    }

    #[test]
    fn mailbox_valid_name_addr() {
        assert!(is_valid_mailbox("User <user@example.com>"));
    }

    #[test]
    fn mailbox_rejects_invalid_or_header_injection_like_input() {
        assert!(!is_valid_mailbox("not-an-email"));
        assert!(!is_valid_mailbox("a@b.com\nBcc: evil@example.com"));
    }
}
