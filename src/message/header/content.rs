use super::{Error, Header, HeaderName, HeaderValue};
use std::{
    fmt::{Display, Formatter as FmtFormatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentTransferEncoding {
    SevenBit,
    QuotedPrintable,
    Base64,
    // 8BITMIME
    EightBit,
    Binary,
}

impl Default for ContentTransferEncoding {
    fn default() -> Self {
        ContentTransferEncoding::SevenBit
    }
}

impl Display for ContentTransferEncoding {
    fn fmt(&self, f: &mut FmtFormatter) -> FmtResult {
        use self::ContentTransferEncoding::*;
        f.write_str(match *self {
            SevenBit => "7bit",
            QuotedPrintable => "quoted-printable",
            Base64 => "base64",
            EightBit => "8bit",
            Binary => "binary",
        })
    }
}

impl FromStr for ContentTransferEncoding {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::ContentTransferEncoding::*;
        match s {
            "7bit" => Ok(SevenBit),
            "quoted-printable" => Ok(QuotedPrintable),
            "base64" => Ok(Base64),
            "8bit" => Ok(EightBit),
            "binary" => Ok(Binary),
            _ => Err(s.into()),
        }
    }
}

impl Header for ContentTransferEncoding {
    fn name() -> &'static HeaderName {
        let name = HeaderName::from_static("Content-Transfer-Encoding");
        &name
    }

    fn decode<'i, I: Iterator<Item = &'i HeaderValue>>(values: &mut I) -> Result<Self, Error> {
        let s = values
            .next()
            .and_then(|v| v.to_str().ok())
            .ok_or(|_| Error::invalid())?;

        s.parse::<ContentTransferEncoding>()
            .map_err(|_| Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = self
            .to_string()
            .parse()
            .expect("HeaderValue is always valid");
        values.extend(std::iter::once(value));
    }
}

#[cfg(test)]
mod test {
    use super::ContentTransferEncoding;
    use hyperx::header::Headers;

    #[test]
    fn format_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.set(ContentTransferEncoding::SevenBit);

        assert_eq!(
            format!("{}", headers),
            "Content-Transfer-Encoding: 7bit\r\n"
        );

        headers.set(ContentTransferEncoding::Base64);

        assert_eq!(
            format!("{}", headers),
            "Content-Transfer-Encoding: base64\r\n"
        );
    }

    #[test]
    fn parse_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.set_raw("Content-Transfer-Encoding", "7bit");

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(&ContentTransferEncoding::SevenBit)
        );

        headers.set_raw("Content-Transfer-Encoding", "base64");

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(&ContentTransferEncoding::Base64)
        );
    }
}
