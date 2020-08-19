<h1 align="center">lettre</h1>
<div align="center">
 <strong>
   A mailer library for Rust
 </strong>
</div>

<br />

<div align="center">
  <a href="https://docs.rs/lettre">
    <img src="https://docs.rs/lettre/badge.svg"
      alt="docs" />
  </a>
  <a href="https://crates.io/crates/lettre">
    <img src="https://img.shields.io/crates/d/lettre.svg"
      alt="downloads" />
  </a>
  <br />
  <a href="https://gitter.im/lettre/lettre">
    <img src="https://badges.gitter.im/lettre/lettre.svg"
      alt="chat on gitter" />
  </a>
  <a href="https://lettre.at">
    <img src="https://img.shields.io/badge/visit-website-blueviolet"
      alt="website" />
  </a>
</div>

---

**NOTE**: this readme refers to the 0.10 version of lettre, which is
still being worked on. The master branch and the alpha releases will see
API breaking changes and some features may be missing or incomplete until
the stable 0.10.0 release is out.
Use the [`v0.9.x`](https://github.com/lettre/lettre/tree/v0.9.x) branch for stable releases.

---

## Features

Lettre provides the following features:

* Multiple transport methods
* Unicode support (for email content and addresses)
* Secure delivery with SMTP using encryption and authentication
* Easy email builders
* Async support (incomplete)

Lettre does not provide (for now):

* Email parsing

## Example

This library requires Rust 1.40 or newer.
To use this library, add the following to your `Cargo.toml`:


```toml
[dependencies]
lettre = "0.10.0-alpha.1"
```

```rust,no_run
use lettre::{Message, SmtpTransport, Transport};

let email = Message::builder()
    .from("NoBody <nobody@domain.tld>".parse().unwrap())
    .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
    .to("Hei <hei@domain.tld>".parse().unwrap())
    .subject("Happy new year")
    .body("Be happy!")
    .unwrap();

// Open a remote connection to gmail
let mailer = SmtpTransport::relay("smtp.gmail.com")
    .unwrap()
    .credentials(("smtp_username", "smtp_password"))
    .build();

// Send the email
match mailer.send(&email) {
    Ok(_) => println!("Email sent successfully!"),
    Err(e) => panic!("Could not send email: {:?}", e),
}
```

## Testing

The `lettre` tests require an open mail server listening locally on port 2525 and the `sendmail` command.

Alternatively only unit tests can be run by doing `cargo test --lib`.

## Code of conduct

Anyone who interacts with Lettre in any space, including but not limited to
this GitHub repository, must follow our [code of conduct](https://github.com/lettre/lettre/blob/master/CODE_OF_CONDUCT.md).

## License

This program is distributed under the terms of the MIT license.

The builder comes from [emailmessage-rs](https://github.com/katyo/emailmessage-rs) by
Kayo, under MIT license.

See [LICENSE](./LICENSE) for details.
