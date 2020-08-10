// cargo run --no-default-features --features "builder smtp-transport tokio02 tokio02-native-tls" --example smtp_async_gmail

use tokio02_crate as tokio;

use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, Message, Tokio02Transport};

// remove before committing
const DEST_ADDR: &str = "";
const PASS: &str = "";

#[tokio02_crate::main]
async fn main() {
    let email = Message::builder()
        .from("Paolo Barbolini <paolo@paolo565.org>".parse().unwrap())
        .to(DEST_ADDR.parse().unwrap())
        .subject("Happy new async year")
        .body("Be happy with async!")
        .unwrap();

    let creds = Credentials::new("paolo@paolo565.org".to_string(), PASS.to_string());
    let transport = AsyncSmtpTransport::relay("smtp.fastmail.com")
        .unwrap()
        .credentials(creds)
        .build();
    transport.send(email).await.unwrap();
}
