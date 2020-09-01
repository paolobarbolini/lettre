use super::{Error, Header, HeaderName, HeaderValue};
use crate::message::utf8_b;

macro_rules! text_header {
    ( $type_name: ident, $header_name: expr ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(pub String);

        impl Header for $type_name {
            fn name() -> &'static HeaderName {
                let name = HeaderName::from_static($header_name);
                &name
            }

            fn decode<'i, I: Iterator<Item = &'i HeaderValue>>(
                values: &mut I,
            ) -> Result<Self, Error> {
                values
                    .next()
                    .and_then(|v| v.to_str().ok())
                    .and_then(utf8_b::decode)
                    .map($type_name)
                    .ok_or_else(Error::invalid)
            }

            fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
                let value = utf8_b::encode(&self.0)
                    .parse()
                    .expect("HeaderValue is always valid");
                values.extend(std::iter::once(value));
            }
        }
    };
}

text_header!(Subject, "Subject");
text_header!(Comments, "Comments");
text_header!(Keywords, "Keywords");
text_header!(InReplyTo, "In-Reply-To");
text_header!(References, "References");
text_header!(MessageId, "Message-Id");

#[cfg(test)]
mod test {
    use super::Subject;
    use hyperx::header::Headers;

    #[test]
    fn format_ascii() {
        let mut headers = Headers::new();
        headers.set(Subject("Sample subject".into()));

        assert_eq!(format!("{}", headers), "Subject: Sample subject\r\n");
    }

    #[test]
    fn format_utf8() {
        let mut headers = Headers::new();
        headers.set(Subject("Тема сообщения".into()));

        assert_eq!(
            format!("{}", headers),
            "Subject: =?utf-8?b?0KLQtdC80LAg0YHQvtC+0LHRidC10L3QuNGP?=\r\n"
        );
    }

    #[test]
    fn parse_ascii() {
        let mut headers = Headers::new();
        headers.set_raw("Subject", "Sample subject");

        assert_eq!(
            headers.get::<Subject>(),
            Some(&Subject("Sample subject".into()))
        );
    }

    #[test]
    fn parse_utf8() {
        let mut headers = Headers::new();
        headers.set_raw(
            "Subject",
            "=?utf-8?b?0KLQtdC80LAg0YHQvtC+0LHRidC10L3QuNGP?=",
        );

        assert_eq!(
            headers.get::<Subject>(),
            Some(&Subject("Тема сообщения".into()))
        );
    }
}
