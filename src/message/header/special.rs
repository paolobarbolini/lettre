use std::fmt::{self, Display, Formatter};

use super::{Error, Header, HeaderName, HeaderValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MimeVersion {
    pub major: u8,
    pub minor: u8,
}

pub const MIME_VERSION_1_0: MimeVersion = MimeVersion { major: 1, minor: 0 };

impl MimeVersion {
    pub fn new(major: u8, minor: u8) -> Self {
        MimeVersion { major, minor }
    }
}

impl Display for MimeVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Default for MimeVersion {
    fn default() -> Self {
        MIME_VERSION_1_0
    }
}

impl Header for MimeVersion {
    fn name() -> &'static HeaderName {
        let name = HeaderName::from_static("MIME-Version");
        &name
    }

    fn decode<'i, I: Iterator<Item = &'i HeaderValue>>(values: &mut I) -> Result<Self, Error> {
        let s = values
            .next()
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Error::invalid())?;

        let mut split = s.split('.');

        let major = split.next().ok_or_else(|| Error::invalid())?;
        let minor = split.next().ok_or_else(|| Error::invalid())?;
        let major = major.parse().map_err(|_| Error::invalid())?;
        let minor = minor.parse().map_err(|_| Error::invalid())?;
        Ok(MimeVersion::new(major, minor))
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
    use super::{MimeVersion, MIME_VERSION_1_0};
    use hyperx::header::Headers;

    #[test]
    fn format_mime_version() {
        let mut headers = Headers::new();

        headers.set(MIME_VERSION_1_0);

        assert_eq!(format!("{}", headers), "MIME-Version: 1.0\r\n");

        headers.set(MimeVersion::new(0, 1));

        assert_eq!(format!("{}", headers), "MIME-Version: 0.1\r\n");
    }

    #[test]
    fn parse_mime_version() {
        let mut headers = Headers::new();

        headers.set_raw("MIME-Version", "1.0");

        assert_eq!(headers.get::<MimeVersion>(), Some(&MIME_VERSION_1_0));

        headers.set_raw("MIME-Version", "0.1");

        assert_eq!(headers.get::<MimeVersion>(), Some(&MimeVersion::new(0, 1)));
    }
}
