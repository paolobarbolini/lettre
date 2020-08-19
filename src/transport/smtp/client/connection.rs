use std::fmt::Display;
use std::io::{self, BufRead, BufReader, Write};
use std::net::ToSocketAddrs;
use std::time::Duration;

#[cfg(feature = "log")]
use log::debug;

use super::{ClientCodec, NetworkStream, TlsParameters};
use crate::transport::smtp::authentication::{Credentials, Mechanism};
use crate::transport::smtp::commands::*;
use crate::transport::smtp::extension::{
    ClientId, Extension, MailBodyParameter, MailParameter, ServerInfo,
};
use crate::transport::smtp::response::{parse_response, Response};
use crate::transport::smtp::Error;
use crate::Envelope;

#[cfg(feature = "log")]
use super::escape_crlf;

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => {
                $client.abort();
                return Err(From::from(err))
            },
        }
    })
);

/// Structure that implements the SMTP client
pub struct SmtpConnection {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: BufReader<NetworkStream>,
    /// Panic state
    panic: bool,
    /// Information about the server
    server_info: ServerInfo,
}

impl SmtpConnection {
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    // FIXME add simple connect and rename this one

    /// Connects to the configured server
    ///
    /// Sends EHLO and parses server information
    pub fn connect<A: ToSocketAddrs>(
        server: A,
        timeout: Option<Duration>,
        hello_name: &ClientId,
        tls_parameters: Option<&TlsParameters>,
    ) -> Result<SmtpConnection, Error> {
        let stream = NetworkStream::connect(server, timeout, tls_parameters)?;
        let stream = BufReader::new(stream);
        let mut conn = SmtpConnection {
            stream,
            panic: false,
            server_info: ServerInfo::default(),
        };
        conn.set_timeout(timeout)?;
        // TODO log
        let _response = conn.read_response()?;

        conn.ehlo(hello_name)?;

        // Print server information
        #[cfg(feature = "log")]
        debug!("server {}", conn.server_info);
        Ok(conn)
    }

    pub fn send(&mut self, envelope: &Envelope, email: &[u8]) -> Result<Response, Error> {
        // Mail
        let mut mail_options = vec![];

        if self.server_info().supports_feature(Extension::EightBitMime) {
            mail_options.push(MailParameter::Body(MailBodyParameter::EightBitMime));
        }
        try_smtp!(
            self.command(Mail::new(envelope.from().cloned(), mail_options)),
            self
        );

        // Recipient
        for to_address in envelope.to() {
            try_smtp!(self.command(Rcpt::new(to_address.clone(), vec![])), self);
        }

        // Data
        try_smtp!(self.command(Data), self);

        // Message content
        let result = try_smtp!(self.message(email), self);
        Ok(result)
    }

    pub fn has_broken(&self) -> bool {
        self.panic
    }

    pub fn can_starttls(&self) -> bool {
        !self.is_encrypted() && self.server_info.supports_feature(Extension::StartTls)
    }

    #[allow(unused_variables)]
    pub fn starttls(
        &mut self,
        tls_parameters: &TlsParameters,
        hello_name: &ClientId,
    ) -> Result<(), Error> {
        if self.server_info.supports_feature(Extension::StartTls) {
            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            {
                try_smtp!(self.command(Starttls), self);
                try_smtp!(self.stream.get_mut().upgrade_tls(tls_parameters), self);
                #[cfg(feature = "log")]
                debug!("connection encrypted");
                // Send EHLO again
                try_smtp!(self.ehlo(hello_name), self);
                Ok(())
            }
            #[cfg(not(any(feature = "native-tls", feature = "rustls-tls")))]
            // This should never happen as `Tls` can only be created
            // when a TLS library is enabled
            unreachable!("TLS support required but not supported");
        } else {
            Err(Error::Client("STARTTLS is not supported on this server"))
        }
    }

    /// Send EHLO and update server info
    fn ehlo(&mut self, hello_name: &ClientId) -> Result<(), Error> {
        let ehlo_response = try_smtp!(
            self.command(Ehlo::new(ClientId::new(hello_name.to_string()))),
            self
        );
        self.server_info = try_smtp!(ServerInfo::from_response(&ehlo_response), self);
        Ok(())
    }

    pub fn quit(&mut self) -> Result<Response, Error> {
        Ok(try_smtp!(self.command(Quit), self))
    }

    pub fn abort(&mut self) {
        // Only try to quit if we are not already broken
        if !self.panic {
            self.panic = true;
            let _ = self.command(Quit);
        }
    }

    /// Sets the underlying stream
    pub fn set_stream(&mut self, stream: NetworkStream) {
        self.stream = BufReader::new(stream);
    }

    /// Tells if the underlying stream is currently encrypted
    pub fn is_encrypted(&self) -> bool {
        self.stream.get_ref().is_encrypted()
    }

    /// Set timeout
    pub fn set_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        self.stream.get_mut().set_read_timeout(duration)?;
        self.stream.get_mut().set_write_timeout(duration)
    }

    /// Checks if the server is connected using the NOOP SMTP command
    pub fn test_connected(&mut self) -> bool {
        self.command(Noop).is_ok()
    }

    /// Sends an AUTH command with the given mechanism, and handles challenge if needed
    pub fn auth(
        &mut self,
        mechanisms: &[Mechanism],
        credentials: &Credentials,
    ) -> Result<Response, Error> {
        let mechanism = self
            .server_info
            .get_auth_mechanism(mechanisms)
            .ok_or(Error::Client(
                "No compatible authentication mechanism was found",
            ))?;

        // Limit challenges to avoid blocking
        let mut challenges = 10;
        let mut response = self.command(Auth::new(mechanism, credentials.clone(), None)?)?;

        while challenges > 0 && response.has_code(334) {
            challenges -= 1;
            response = try_smtp!(
                self.command(Auth::new_from_response(
                    mechanism,
                    credentials.clone(),
                    &response,
                )?),
                self
            );
        }

        if challenges == 0 {
            Err(Error::ResponseParsing("Unexpected number of challenges"))
        } else {
            Ok(response)
        }
    }

    /// Sends the message content
    pub fn message(&mut self, message: &[u8]) -> Result<Response, Error> {
        let mut out_buf: Vec<u8> = vec![];
        let mut codec = ClientCodec::new();
        codec.encode(message, &mut out_buf);
        self.write(out_buf.as_slice())?;
        self.write(b"\r\n.\r\n")?;
        self.read_response()
    }

    /// Sends an SMTP command
    pub fn command<C: Display>(&mut self, command: C) -> Result<Response, Error> {
        self.write(command.to_string().as_bytes())?;
        self.read_response()
    }

    /// Writes a string to the server
    fn write(&mut self, string: &[u8]) -> Result<(), Error> {
        self.stream.get_mut().write_all(string)?;
        self.stream.get_mut().flush()?;

        #[cfg(feature = "log")]
        debug!("Wrote: {}", escape_crlf(&String::from_utf8_lossy(string)));
        Ok(())
    }

    /// Gets the SMTP response
    pub fn read_response(&mut self) -> Result<Response, Error> {
        let mut buffer = String::with_capacity(100);

        while self.stream.read_line(&mut buffer)? > 0 {
            #[cfg(feature = "log")]
            debug!("<< {}", escape_crlf(&buffer));
            match parse_response(&buffer) {
                Ok((_remaining, response)) => {
                    if response.is_positive() {
                        return Ok(response);
                    }

                    return Err(response.into());
                }
                Err(nom::Err::Failure(e)) => {
                    return Err(Error::Parsing(e.1));
                }
                Err(nom::Err::Incomplete(_)) => { /* read more */ }
                Err(nom::Err::Error(e)) => {
                    return Err(Error::Parsing(e.1));
                }
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, "incomplete").into())
    }
}
