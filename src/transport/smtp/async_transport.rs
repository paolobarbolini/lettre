use async_trait::async_trait;

use super::client::AsyncSmtpConnection;
use super::{
    ClientId, Credentials, Error, Mechanism, Response, SmtpInfo, Tls, TlsParameters,
    SUBMISSIONS_PORT,
};
use crate::{Envelope, Tokio02Transport};

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AsyncSmtpTransport {
    // TODO: pool
    inner: AsyncSmtpClient,
}

#[async_trait]
impl Tokio02Transport for AsyncSmtpTransport {
    type Ok = Response;
    type Error = Error;

    /// Sends an email
    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut conn = self.inner.connection().await?;

        let result = conn.send(envelope, email).await?;

        conn.quit().await?;

        Ok(result)
    }
}

impl AsyncSmtpTransport {
    /// Simple and secure transport, should be used when possible.
    /// Creates an encrypted transport over submissions port, using the provided domain
    /// to validate TLS certificates.
    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    pub fn relay(relay: &str) -> Result<AsyncSmtpTransportBuilder, Error> {
        let tls_parameters = TlsParameters::new_tokio02(relay.into())?;

        Ok(Self::builder_dangerous(relay)
            .port(SUBMISSIONS_PORT)
            .tls(Tls::Wrapper(tls_parameters)))
    }

    /// Creates a new local SMTP client to port 25
    ///
    /// Shortcut for local unencrypted relay (typical local email daemon that will handle relaying)
    pub fn unencrypted_localhost() -> AsyncSmtpTransport {
        Self::builder_dangerous("localhost").build()
    }

    /// Creates a new SMTP client
    ///
    /// Defaults are:
    ///
    /// * No authentication
    /// * No TLS
    /// * Port 25
    ///
    /// Consider using [`AsyncSmtpTransport::relay`] instead, if possible.
    pub fn builder_dangerous<T: Into<String>>(server: T) -> AsyncSmtpTransportBuilder {
        let mut new = SmtpInfo::default();
        new.server = server.into();
        AsyncSmtpTransportBuilder { info: new }
    }
}

/// Contains client configuration
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AsyncSmtpTransportBuilder {
    info: SmtpInfo,
}

/// Builder for the SMTP `AsyncSmtpTransport`
impl AsyncSmtpTransportBuilder {
    /// Set the name used during EHLO
    pub fn hello_name(mut self, name: ClientId) -> Self {
        self.info.hello_name = name;
        self
    }

    /// Set the authentication mechanism to use
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.info.credentials = Some(credentials);
        self
    }

    /// Set the authentication mechanism to use
    pub fn authentication(mut self, mechanisms: Vec<Mechanism>) -> Self {
        self.info.authentication = mechanisms;
        self
    }

    /// Set the port to use
    pub fn port(mut self, port: u16) -> Self {
        self.info.port = port;
        self
    }

    /// Set the TLS settings to use
    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    pub fn tls(mut self, tls: Tls) -> Self {
        self.info.tls = tls;
        self
    }

    /// Build the client
    fn build_client(self) -> AsyncSmtpClient {
        AsyncSmtpClient { info: self.info }
    }

    /// Build the transport (with default pool if enabled)
    pub fn build(self) -> AsyncSmtpTransport {
        let client = self.build_client();
        AsyncSmtpTransport { inner: client }
    }
}

/// Build client
#[derive(Clone)]
pub struct AsyncSmtpClient {
    info: SmtpInfo,
}

impl AsyncSmtpClient {
    /// Creates a new connection directly usable to send emails
    ///
    /// Handles encryption and authentication
    pub async fn connection(&self) -> Result<AsyncSmtpConnection, Error> {
        #[allow(clippy::match_single_binding)]
        let tls = match self.info.tls {
            #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
            Tls::Wrapper(ref tls_parameters) => Some(tls_parameters.clone()),
            _ => None,
        };

        let addr = (self.info.server.as_ref(), self.info.port);
        let mut conn =
            AsyncSmtpConnection::connect_tokio02(addr, &self.info.hello_name, tls).await?;

        #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
        match self.info.tls {
            Tls::Opportunistic(ref tls_parameters) => {
                if conn.can_starttls() {
                    conn.starttls(tls_parameters.clone(), &self.info.hello_name)
                        .await?;
                }
            }
            Tls::Required(ref tls_parameters) => {
                conn.starttls(tls_parameters.clone(), &self.info.hello_name)
                    .await?;
            }
            _ => (),
        }

        if let Some(credentials) = &self.info.credentials {
            conn.auth(&self.info.authentication, &credentials).await?;
        }

        Ok(conn)
    }
}
