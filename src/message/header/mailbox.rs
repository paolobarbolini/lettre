use super::{Error, Header, HeaderName, HeaderValue};
use crate::message::{
    mailbox::{Mailbox, Mailboxes},
    utf8_b,
};

/// Header which can contains multiple mailboxes
pub trait MailboxesHeader {
    fn join_mailboxes(&mut self, other: Self);
}

macro_rules! mailbox_header {
    ($(#[$doc:meta])*($type_name: ident, $header_name: expr)) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(pub Mailbox);

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
                    .and_then(|s|Mailboxes::parse(s).ok())
                    .and_then(|mbs| {
                        mbs.into_single().ok_or_else(||Error::invalid())
                    }).map($type_name)
                    .ok_or_else(||Error::invalid())
            }

            fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
                let value = self.0.recode_name(utf8_b::encode).to_string()
                    .parse()
                    .expect("HeaderValue is always valid");
                values.extend(std::iter::once(value));
            }
        }
    };
}

macro_rules! mailboxes_header {
    ($(#[$doc:meta])*($type_name: ident, $header_name: expr)) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(pub Mailboxes);

        impl MailboxesHeader for $type_name {
            fn join_mailboxes(&mut self, other: Self) {
                self.0.extend(other.0);
            }
        }

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
                    .and_then(Ma)
                    .ok_or_else(||Error::invalid())
            }

            fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
                let mailboxes = Mailboxes::from(
                    self.0.iter().map(|mb| mb.recode_name(utf8_b::encode))
                        .collect::<Vec<_>>(),
                );

                let value = mailboxes
                    .to_string()
                    .parse()
                    .expect("HeaderValue is always valid");
                values.extend(std::iter::once(value));
            }
        }
    };
}

mailbox_header! {
    /**

    `Sender` header

    This header contains [`Mailbox`][self::Mailbox] associated with sender.

    ```no_test
    header::Sender("Mr. Sender <sender@example.com>".parse().unwrap())
    ```
     */
    (Sender, "Sender")
}

mailboxes_header! {
    /**

    `From` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (From, "From")
}

mailboxes_header! {
    /**

    `Reply-To` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (ReplyTo, "Reply-To")
}

mailboxes_header! {
    /**

    `To` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (To, "To")
}

mailboxes_header! {
    /**

    `Cc` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (Cc, "Cc")
}

mailboxes_header! {
    /**

    `Bcc` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (Bcc, "Bcc")
}

#[cfg(test)]
mod test {
    use super::{From, Mailbox, Mailboxes};
    use hyperx::header::Headers;

    #[test]
    fn format_single_without_name() {
        let from = Mailboxes::new().with("kayo@example.com".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(format!("{}", headers), "From: kayo@example.com\r\n");
    }

    #[test]
    fn format_single_with_name() {
        let from = Mailboxes::new().with("K. <kayo@example.com>".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(format!("{}", headers), "From: K. <kayo@example.com>\r\n");
    }

    #[test]
    fn format_multi_without_name() {
        let from = Mailboxes::new()
            .with("kayo@example.com".parse().unwrap())
            .with("pony@domain.tld".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(
            format!("{}", headers),
            "From: kayo@example.com, pony@domain.tld\r\n"
        );
    }

    #[test]
    fn format_multi_with_name() {
        let from = vec![
            "K. <kayo@example.com>".parse().unwrap(),
            "Pony P. <pony@domain.tld>".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set(From(from.into()));

        assert_eq!(
            format!("{}", headers),
            "From: K. <kayo@example.com>, Pony P. <pony@domain.tld>\r\n"
        );
    }

    #[test]
    fn format_single_with_utf8_name() {
        let from = vec!["Кайо <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set(From(from.into()));

        assert_eq!(
            format!("{}", headers),
            "From: =?utf-8?b?0JrQsNC50L4=?= <kayo@example.com>\r\n"
        );
    }

    #[test]
    fn parse_single_without_name() {
        let from = vec!["kayo@example.com".parse().unwrap()].into();

        let mut headers = Headers::new();
        headers.set_raw("From", "kayo@example.com");

        assert_eq!(headers.get::<From>(), Some(&From(from)));
    }

    #[test]
    fn parse_single_with_name() {
        let from = vec!["K. <kayo@example.com>".parse().unwrap()].into();

        let mut headers = Headers::new();
        headers.set_raw("From", "K. <kayo@example.com>");

        assert_eq!(headers.get::<From>(), Some(&From(from)));
    }

    #[test]
    fn parse_multi_without_name() {
        let from: Vec<Mailbox> = vec![
            "kayo@example.com".parse().unwrap(),
            "pony@domain.tld".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set_raw("From", "kayo@example.com, pony@domain.tld");

        assert_eq!(headers.get::<From>(), Some(&From(from.into())));
    }

    #[test]
    fn parse_multi_with_name() {
        let from: Vec<Mailbox> = vec![
            "K. <kayo@example.com>".parse().unwrap(),
            "Pony P. <pony@domain.tld>".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set_raw("From", "K. <kayo@example.com>, Pony P. <pony@domain.tld>");

        assert_eq!(headers.get::<From>(), Some(&From(from.into())));
    }

    #[test]
    fn parse_single_with_utf8_name() {
        let from: Vec<Mailbox> = vec!["Кайо <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set_raw("From", "=?utf-8?b?0JrQsNC50L4=?= <kayo@example.com>");

        assert_eq!(headers.get::<From>(), Some(&From(from.into())));
    }
}
