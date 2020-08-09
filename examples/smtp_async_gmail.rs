// cargo run --no-default-features --features "builder smtp-transport tokio02 tokio02-native-tls" --example smtp_async_gmail

use tokio02_crate as tokio;

use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::authentication::Mechanism;
use lettre::transport::smtp::client::{AsyncSmtpConnection, TlsParameters};
use lettre::transport::smtp::extension::ClientId;
use lettre::Message;

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

    println!("1");
    let tls_parameters = TlsParameters::new("smtp.fastmail.com".to_string()).unwrap();
    println!("3");
    let mut conn = AsyncSmtpConnection::connect("smtp.fastmail.com:465", &ClientId::hostname(), Some(tls_parameters))
        .await
        .unwrap();
    println!("4");
    conn.auth(&[Mechanism::Plain], &creds).await.unwrap();
    println!("5");
    conn.send(email.envelope(), &email.formatted())
        .await
        .unwrap();
    println!("6");
}
