//! ### Sending Messages
//!
//! This section explains how to manipulate emails you have created.
//!
//! This mailer contains several different transports for your emails. To be sendable, the
//! emails have to implement `Email`, which is the case for emails created with `lettre::builder`.
//!
//! The following transports are available:
//!
//! * The `SmtpTransport` uses the SMTP protocol to send the message over the network. It is
//!   the preferred way of sending emails.
//! * The `SendmailTransport` uses the sendmail command to send messages. It is an alternative to
//!   the SMTP transport.
//! * The `FileTransport` creates a file containing the email content to be sent. It can be used
//!   for debugging or if you want to keep all sent emails.
//! * The `StubTransport` is useful for debugging, and only prints the content of the email in the
//!   logs.

use std::{error::Error as StdError, fmt};

#[cfg(any(feature = "async-std1", feature = "tokio02"))]
use async_trait::async_trait;

use crate::address::Envelope;
use crate::Message;

#[cfg(feature = "file-transport")]
pub mod file;
#[cfg(feature = "sendmail-transport")]
pub mod sendmail;
#[cfg(feature = "smtp-transport")]
pub mod smtp;
pub mod stub;

/// Blocking Transport method for emails
pub trait Transport {
    /// Result types for the transport
    type Ok: fmt::Debug;
    type Error: StdError;

    /// Sends the email
    #[cfg(feature = "builder")]
    fn send(&self, message: &Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        self.send_raw(message.envelope(), &raw)
    }

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

/// async-std 1.x based Transport method for emails
#[cfg(feature = "async-std1")]
#[async_trait]
pub trait AsyncStd1Transport {
    /// Result types for the transport
    type Ok: fmt::Debug;
    type Error: StdError;

    /// Sends the email
    #[cfg(feature = "builder")]
    // TODO take &Message
    async fn send(&self, message: Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        let envelope = message.envelope();
        self.send_raw(&envelope, &raw).await
    }

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

/// tokio 0.2.x based Transport method for emails
#[cfg(feature = "tokio02")]
#[async_trait]
pub trait Tokio02Transport {
    /// Result types for the transport
    type Ok: fmt::Debug;
    type Error: StdError;

    /// Sends the email
    #[cfg(feature = "builder")]
    // TODO take &Message
    async fn send(&self, message: Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        let envelope = message.envelope();
        self.send_raw(&envelope, &raw).await
    }

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}
