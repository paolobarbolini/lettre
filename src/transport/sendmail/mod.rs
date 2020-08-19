//! The sendmail transport sends the email using the local sendmail command.
//!
//! #### Sendmail Transport
//!
//! The sendmail transport sends the email using the local sendmail command.
//!
//! ```rust,no_run
//! # #[cfg(feature = "sendmail-transport")]
//! # {
//! use lettre::{Message, Envelope, Transport, SendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse().unwrap())
//!     .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
//!     .to("Hei <hei@domain.tld>".parse().unwrap())
//!     .subject("Happy new year")
//!     .body("Be happy!")
//!     .unwrap();
//!
//! let sender = SendmailTransport::new();
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # }
//! ```

use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};

#[cfg(any(feature = "async-std1", feature = "tokio02"))]
use async_trait::async_trait;

pub use self::error::Error;
#[cfg(feature = "async-std1")]
use crate::AsyncStd1Transport;
#[cfg(feature = "tokio02")]
use crate::Tokio02Transport;
use crate::{Envelope, Transport};

mod error;

const DEFAUT_SENDMAIL: &str = "/usr/sbin/sendmail";

/// Sends an email using the `sendmail` command
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendmailTransport {
    command: OsString,
}

impl SendmailTransport {
    /// Creates a new transport with the default `/usr/sbin/sendmail` command
    pub fn new() -> SendmailTransport {
        SendmailTransport {
            command: DEFAUT_SENDMAIL.into(),
        }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<OsString>>(command: S) -> SendmailTransport {
        SendmailTransport {
            command: command.into(),
        }
    }

    fn command(&self, envelope: &Envelope) -> Command {
        let mut c = Command::new(&self.command);
        c.arg("-i")
            .arg("-f")
            .arg(envelope.from().map(|f| f.as_ref()).unwrap_or("\"\""))
            .args(envelope.to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        c
    }

    #[cfg(feature = "tokio02")]
    fn tokio02_command(&self, envelope: &Envelope) -> tokio02_crate::process::Command {
        use tokio02_crate::process::Command;

        let mut c = Command::new(&self.command);
        c.kill_on_drop(true);
        c.arg("-i")
            .arg("-f")
            .arg(envelope.from().map(|f| f.as_ref()).unwrap_or("\"\""))
            .args(envelope.to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        c
    }
}

impl Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        // Spawn the sendmail command
        let mut process = self.command(envelope).spawn()?;

        process.stdin.as_mut().unwrap().write_all(email)?;
        let output = process.wait_with_output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(error::Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}

#[cfg(feature = "async-std1")]
#[async_trait]
impl AsyncStd1Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut command = self.command(envelope);
        let email = email.to_vec();

        // TODO: Convert to real async, once async-std has a process implementation.
        let output = async_std::task::spawn_blocking(move || {
            // Spawn the sendmail command
            let mut process = command.spawn()?;

            process.stdin.as_mut().unwrap().write_all(&email)?;
            process.wait_with_output()
        })
        .await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}

#[cfg(feature = "tokio02")]
#[async_trait]
impl Tokio02Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio02_crate::io::AsyncWriteExt;

        let mut command = self.tokio02_command(envelope);

        // Spawn the sendmail command
        let mut process = command.spawn()?;

        process.stdin.as_mut().unwrap().write_all(&email).await?;
        let output = process.wait_with_output().await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}
