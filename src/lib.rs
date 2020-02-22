//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! This mailer contains the available transports for your emails.
//!

#![doc(html_root_url = "https://docs.rs/lettre/0.10.0")]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[cfg(feature = "builder")]
pub mod builder;
pub mod error;
#[cfg(feature = "file-transport")]
pub mod file;
#[cfg(feature = "sendmail-transport")]
pub mod sendmail;
#[cfg(feature = "smtp-transport")]
pub mod smtp;
pub mod stub;

#[cfg(feature = "builder")]
use crate::builder::EmailBuilder;
use crate::error::EmailResult;
use crate::error::Error;
#[cfg(feature = "file-transport")]
pub use crate::file::FileTransport;
#[cfg(feature = "sendmail-transport")]
pub use crate::sendmail::SendmailTransport;
#[cfg(feature = "smtp-transport")]
pub use crate::smtp::client::net::ClientTlsParameters;
#[cfg(all(feature = "smtp-transport", feature = "connection-pool"))]
pub use crate::smtp::r2d2::SmtpConnectionManager;
#[cfg(feature = "smtp-transport")]
pub use crate::smtp::{ClientSecurity, SmtpClient, SmtpTransport};
use fast_chemail::is_valid_email;
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};
use std::str::{FromStr, Utf8Error};

/// Email address
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(address: String) -> EmailResult<EmailAddress> {
        if !EmailAddress::is_valid(&address) {
            return Err(Error::InvalidEmailAddress);
        }
        Ok(EmailAddress(address))
    }

    pub fn is_valid(addr: &str) -> bool {
        is_valid_email(addr) || addr.ends_with("localhost")
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for EmailAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EmailAddress::new(s.to_string())
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for EmailAddress {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Simple email envelope representation
///
/// We only accept mailboxes, and do not support source routes (as per RFC).
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Envelope {
    /// The envelope recipients' addresses
    ///
    /// This can not be empty.
    forward_path: Vec<EmailAddress>,
    /// The envelope sender address
    reverse_path: Option<EmailAddress>,
}

impl Envelope {
    /// Creates a new envelope, which may fail if `to` is empty.
    pub fn new(from: Option<EmailAddress>, to: Vec<EmailAddress>) -> EmailResult<Envelope> {
        if to.is_empty() {
            return Err(Error::MissingTo);
        }
        Ok(Envelope {
            forward_path: to,
            reverse_path: from,
        })
    }

    /// Destination addresses of the envelope
    pub fn to(&self) -> &[EmailAddress] {
        self.forward_path.as_slice()
    }

    /// Source address of the envelope
    pub fn from(&self) -> Option<&EmailAddress> {
        self.reverse_path.as_ref()
    }
}

/// Sendable email structure
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Email {
    envelope: Envelope,
    message_id: String,
    message: Vec<u8>,
}

impl Email {
    /// Creates a new email builder
    #[cfg(feature = "builder")]
    pub fn builder() -> EmailBuilder {
        EmailBuilder::new()
    }

    pub fn new(envelope: Envelope, message_id: String, message: Vec<u8>) -> Email {
        Email {
            envelope,
            message_id,
            message,
        }
    }

    pub fn new_with_reader(
        envelope: Envelope,
        message_id: String,
        message: &mut dyn Read,
    ) -> Result<Email, io::Error> {
        let mut buf = Vec::new();
        message.read_to_end(&mut buf)?;

        Ok(Email {
            envelope,
            message_id,
            message: buf,
        })
    }

    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    pub fn message(&self) -> &[u8] {
        self.message.as_slice()
    }

    pub fn message_as_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.message())
    }
}

#[cfg(feature = "serde")]
impl Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Email", 3)?;
        state.serialize_field("envelope", &self.envelope.clone())?;
        state.serialize_field("message_id", &self.message_id)?;
        state.serialize_field("message", self.message_as_str().unwrap())?;
        state.end()
    }
}

/// Transport method for emails
pub trait Transport<'a> {
    /// Result type for the transport
    type Result;

    /// Sends the email
    fn send<E: Into<Email>>(&mut self, email: E) -> Self::Result;
}
