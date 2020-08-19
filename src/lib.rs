//! Lettre is an email library that allows creating and sending messages. It provides:
//!
//! * An easy to use email builder
//! * Pluggable email transports
//! * Unicode support
//! * Secure defaults
//!
//! Lettre requires Rust 1.40 or newer.
//!
//! ## Optional features
//!
//! * **builder**: Message builder
//! * **file-transport**: Transport that write messages into a file
//! * **smtp-transport**: Transport over SMTP
//! * **sendmail-transport**: Transport over SMTP
//! * **rustls-tls**: TLS support with the `rustls` crate
//! * **native-tls**: TLS support with the `native-tls` crate
//! * **r2d2**: Connection pool for SMTP transport
//! * **log**: Logging using the `log` crate
//! * **serde**: Serialization/Deserialization of entities
//! * **hostname**: Ability to try to use actual hostname in SMTP transaction

#![doc(html_root_url = "https://docs.rs/lettre/0.10.0")]
#![doc(html_favicon_url = "https://lettre.at/favicon.png")]
#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/15113230?v=4")]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unsafe_code
)]
// TODO: remove once MSRV is >=1.42.0
#![allow(clippy::match_like_matches_macro)]

pub use crate::address::{Address, AddressError, Envelope};
#[doc(inline)]
#[cfg(feature = "builder")]
pub use crate::message::{Mailbox, Mailboxes, Message};
#[doc(inline)]
#[cfg(feature = "file-transport")]
pub use crate::transport::file::FileTransport;
#[doc(inline)]
#[cfg(feature = "sendmail-transport")]
pub use crate::transport::sendmail::SendmailTransport;
#[doc(inline)]
#[cfg(feature = "smtp-transport")]
pub use crate::transport::smtp::SmtpTransport;
#[doc(inline)]
#[cfg(feature = "async-std1")]
pub use crate::transport::AsyncStd1Transport;
#[doc(inline)]
#[cfg(feature = "tokio02")]
pub use crate::transport::Tokio02Transport;
#[doc(inline)]
pub use crate::transport::Transport;

mod address;
#[cfg(feature = "builder")]
pub mod message;
pub mod transport;
