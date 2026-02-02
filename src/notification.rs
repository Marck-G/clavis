use lettre::Transport;
use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, transport::smtp::authentication::Credentials};
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct SMTPConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub activate: bool, // if smt is activated
}

impl SMTPConfig {
    pub fn load() -> Self {
        // check if exist load else create default struct and set activated to false
        debug!("Environemnte dumb: {:#?}", std::env::vars());
        if std::env::var("SMTP_HOST").is_err()
            || std::env::var("SMTP_PORT").is_err()
            || std::env::var("SMTP_USERNAME").is_err()
            || std::env::var("SMTP_PASSWORD").is_err()
        {
            return Self {
                host: "".to_string(),
                port: 0,
                username: "".to_string(),
                password: "".to_string(),
                activate: false,
            };
        }
        let host = std::env::var("SMTP_HOST").expect("SMTP_HOST not set");
        let port = std::env::var("SMTP_PORT")
            .expect("SMTP_PORT not set")
            .parse()
            .expect("SMTP_PORT is not a number");
        let username = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set");
        let password = std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not set");
        Self {
            host,
            port,
            username,
            password,
            activate: true,
        }
    }
}

pub fn send_email(to: String) -> Result<(), String> {
    let smtp_config = SMTPConfig::load();
    debug!("Loaded smtp config {:#?}", smtp_config);
    if !smtp_config.activate {
        warn!("Not smtp config found");
        return Err("SMTP is not activated".to_string());
    }
    let from = std::env::var("SMTP_FROM").expect("SMTP_FROM not set");
    let to = to;
    let subject = "Password Change";
    let body = r#"
	<div style="width: 80%; margin: 0 auto; border: 1px solid #ccc; padding: 20px;">
	<h1>Password changed</h1>
	<p>
	You requested a password change
	</p>
	</div>
	"#;
    info!("Creating the email");
    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .header(ContentType::TEXT_HTML)
        .subject(subject)
        .body(body.to_string())
        .unwrap();
    info!("creating the transport");
    let sender = SmtpTransport::starttls_relay(&smtp_config.host)
        .unwrap()
        .port(smtp_config.port)
        .credentials(Credentials::new(smtp_config.username, smtp_config.password))
        .build();
    info!("Sending email");
    sender.send(&email).unwrap();
    Ok(())
}
