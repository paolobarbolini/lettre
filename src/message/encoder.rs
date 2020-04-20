use crate::message::header::ContentTransferEncoding;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{
    cmp::min,
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

/// Content encoding error
#[derive(Debug, Clone)]
pub enum EncoderError<E> {
    Source(E),
    Coding,
}

impl<E> Error for EncoderError<E> where E: Debug + Display {}

impl<E> Display for EncoderError<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            EncoderError::Source(error) => write!(f, "Source error: {}", error),
            EncoderError::Coding => f.write_str("Coding error"),
        }
    }
}

/// Encoder trait
pub trait EncoderCodec<B: Buf>: Send {
    /// Encode chunk of data
    fn encode_chunk(&mut self, input: B) -> Result<Bytes, ()>;

    /// Encode end of stream
    ///
    /// This proposed to use for stateful encoders like *base64*.
    fn finish_chunk(&mut self) -> Result<Bytes, ()> {
        Ok(Bytes::new())
    }

    /// Encode all data
    fn encode_all(&mut self, source: B) -> Result<Bytes, ()> {
        let chunk = self.encode_chunk(source)?;
        let end = self.finish_chunk()?;

        Ok(if end.is_empty() {
            chunk
        } else {
            let mut chunk = BytesMut::from(chunk.bytes());
            chunk.put(end);
            chunk.freeze()
        })
    }
}

/// 7bit codec
///
struct SevenBitCodec {
    line_wrapper: EightBitCodec,
}

impl SevenBitCodec {
    pub fn new() -> Self {
        SevenBitCodec {
            line_wrapper: EightBitCodec::new(),
        }
    }
}

impl<B> EncoderCodec<B> for SevenBitCodec
where
    B: Buf,
{
    fn encode_chunk(&mut self, chunk: B) -> Result<Bytes, ()> {
        if chunk.bytes().iter().all(u8::is_ascii) {
            self.line_wrapper.encode_chunk(chunk)
        } else {
            Err(())
        }
    }
}

/// Quoted-Printable codec
///
struct QuotedPrintableCodec();

impl QuotedPrintableCodec {
    pub fn new() -> Self {
        QuotedPrintableCodec()
    }
}

impl<B> EncoderCodec<B> for QuotedPrintableCodec
where
    B: Buf,
{
    fn encode_chunk(&mut self, chunk: B) -> Result<Bytes, ()> {
        Ok(quoted_printable::encode(chunk.bytes()).into())
    }
}

/// Base64 codec
///
struct Base64Codec {
    line_wrapper: EightBitCodec,
    last_padding: Bytes,
}

impl Base64Codec {
    pub fn new() -> Self {
        Base64Codec {
            line_wrapper: EightBitCodec::new().with_limit(78 - 2),
            last_padding: Bytes::new(),
        }
    }
}

impl<B> EncoderCodec<B> for Base64Codec
where
    B: Buf,
{
    fn encode_chunk(&mut self, mut chunk: B) -> Result<Bytes, ()> {
        let in_len = self.last_padding.len() + chunk.remaining();
        let out_len = in_len * 4 / 3;

        let mut out = BytesMut::with_capacity(out_len);

        if !self.last_padding.is_empty() {
            let mut src = BytesMut::with_capacity(3);
            let len = min(chunk.remaining(), 3 - self.last_padding.len());

            src.put(&mut self.last_padding);
            src.put(&chunk.bytes()[..len]);

            // encode beginning
            out.resize(src.len() * 4 / 3, 0);
            base64::encode_config_slice(&src, base64::STANDARD, &mut out[..]);
            chunk.advance(len);
        };

        let len = chunk.remaining() - (chunk.remaining() % 3);
        if len > 0 {
            // encode chunk
            out.resize(out.len() + len * 4 / 3, 0);
            base64::encode_config_slice(&chunk.bytes()[..len], base64::STANDARD, &mut out[..]);
            chunk.advance(len);
        }

        // update last padding
        self.last_padding = chunk.to_bytes();
        self.line_wrapper.encode_chunk(&mut out)
    }

    fn finish_chunk(&mut self) -> Result<Bytes, ()> {
        let mut out: [u8; 4] = [0; 4];
        let written = base64::encode_config_slice(&self.last_padding, base64::STANDARD, &mut out);

        self.line_wrapper.encode_chunk(&mut &out[..written])
    }
}

/// 8bit codec
///
struct EightBitCodec {
    max_length: usize,
    line_bytes: usize,
}

const DEFAULT_MAX_LINE_LENGTH: usize = 1000 - 2;

impl EightBitCodec {
    pub fn new() -> Self {
        EightBitCodec {
            max_length: DEFAULT_MAX_LINE_LENGTH,
            line_bytes: 0,
        }
    }

    pub fn with_limit(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }
}

impl<B> EncoderCodec<B> for EightBitCodec
where
    B: Buf,
{
    fn encode_chunk(&mut self, mut chunk: B) -> Result<Bytes, ()> {
        let mut out = BytesMut::with_capacity(chunk.remaining() + 20);
        while chunk.has_remaining() {
            let line_break = chunk.bytes().iter().position(|b| *b == b'\n');
            let mut split_pos = if let Some(line_break) = line_break {
                line_break
            } else {
                chunk.remaining()
            };
            let max_length = self.max_length - self.line_bytes;
            if split_pos < max_length {
                // advance line bytes
                self.line_bytes += split_pos;
            } else {
                split_pos = max_length;
                // reset line bytes
                self.line_bytes = 0;
            };
            let has_remaining = split_pos < chunk.remaining();
            //let mut taken = chunk.take(split_pos);
            out.reserve(split_pos + if has_remaining { 2 } else { 0 });
            //out.put(&mut taken);
            out.put(&chunk.bytes()[..split_pos]);
            if has_remaining {
                out.put_slice(b"\r\n");
            }
            chunk.advance(split_pos);
            //src = taken.into_inner();
        }
        Ok(out.freeze())
    }
}

/// Binary codec
///
struct BinaryCodec;

impl BinaryCodec {
    pub fn new() -> Self {
        BinaryCodec
    }
}

impl<B> EncoderCodec<B> for BinaryCodec
where
    B: Buf,
{
    fn encode_chunk(&mut self, mut chunk: B) -> Result<Bytes, ()> {
        Ok(chunk.to_bytes())
    }
}

pub fn codec<B: Buf>(encoding: Option<&ContentTransferEncoding>) -> Box<dyn EncoderCodec<B>> {
    use self::ContentTransferEncoding::*;
    if let Some(encoding) = encoding {
        match encoding {
            SevenBit => Box::new(SevenBitCodec::new()),
            QuotedPrintable => Box::new(QuotedPrintableCodec::new()),
            Base64 => Box::new(Base64Codec::new()),
            EightBit => Box::new(EightBitCodec::new()),
            Binary => Box::new(BinaryCodec::new()),
        }
    } else {
        Box::new(BinaryCodec::new())
    }
}

#[cfg(test)]
mod test {
    use super::{
        Base64Codec, BinaryCodec, EightBitCodec, EncoderCodec, QuotedPrintableCodec, SevenBitCodec,
    };
    use std::str::from_utf8;

    #[test]
    fn seven_bit_encode() {
        let mut c = SevenBitCodec::new();

        assert_eq!(
            c.encode_chunk("Hello, world!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, world!".into()))
        );

        assert_eq!(
            c.encode_chunk("Hello, мир!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Err(())
        );
    }

    #[test]
    fn quoted_printable_encode() {
        let mut c = QuotedPrintableCodec::new();

        assert_eq!(
            c.encode_chunk("Привет, мир!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok(
                "=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!".into()
            ))
        );

        assert_eq!(c.encode_chunk("Текст письма в уникоде".as_bytes())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5".into())));
    }

    #[test]
    fn base64_encode() {
        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_all("Привет, мир!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("0J/RgNC40LLQtdGCLCDQvNC40YAh".into()))
        );

        assert_eq!(
            c.encode_all("Текст письма в уникоде подлиннее.".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok(concat!(
                "0KLQtdC60YHRgiDQv9C40YHRjNC80LAg0LIg0YPQvdC40LrQ\r\n",
                "vtC00LUg0L/QvtC00LvQuNC90L3QtdC1Lg=="
            )
            .into()))
        );
    }

    #[test]
    fn base64_encode_all() {
        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_all(
                "Ну прямо супер-длинный текст письма в уникоде, который уж точно ну никак не поместиться в 78 байт, как ни крути, я гарантирую.".as_bytes()
            ).map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok(
                concat!("0J3RgyDQv9GA0Y/QvNC+INGB0YPQv9C10YAt0LTQu9C40L3QvdGL0Lkg0YLQtdC60YHRgiDQv9C4\r\n",
                        "0YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LUsINC60L7RgtC+0YDRi9C5INGD0LYg0YLQvtGH0L3Q\r\n",
                        "viDQvdGDINC90LjQutCw0Log0L3QtSDQv9C+0LzQtdGB0YLQuNGC0YzRgdGPINCyIDc4INCx0LDQ\r\n",
                        "udGCLCDQutCw0Log0L3QuCDQutGA0YPRgtC4LCDRjyDQs9Cw0YDQsNC90YLQuNGA0YPRji4=").into()
            ))
        );

        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_all(
                "Ну прямо супер-длинный текст письма в уникоде, который уж точно ну никак не поместиться в 78 байт, как ни крути, я гарантирую это."
                    .as_bytes()
            ).map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok(
                concat!("0J3RgyDQv9GA0Y/QvNC+INGB0YPQv9C10YAt0LTQu9C40L3QvdGL0Lkg0YLQtdC60YHRgiDQv9C4\r\n",
                        "0YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LUsINC60L7RgtC+0YDRi9C5INGD0LYg0YLQvtGH0L3Q\r\n",
                        "viDQvdGDINC90LjQutCw0Log0L3QtSDQv9C+0LzQtdGB0YLQuNGC0YzRgdGPINCyIDc4INCx0LDQ\r\n",
                        "udGCLCDQutCw0Log0L3QuCDQutGA0YPRgtC4LCDRjyDQs9Cw0YDQsNC90YLQuNGA0YPRjiDRjdGC\r\n",
                        "0L4u").into()
            ))
        );
    }
    /*
    #[test]
    fn base64_encode_chunked() {
        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_chunk("Chunk.".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Q2h1bmsu".into()))
        );

        assert_eq!(
            c.finish_chunk()
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("".into()))
        );

        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_chunk("Chunk".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Q2h1".into()))
        );

        assert_eq!(
            c.finish_chunk()
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("bms=".into()))
        );

        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_chunk("Chun".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Q2h1".into()))
        );

        assert_eq!(
            c.finish_chunk()
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("bg==".into()))
        );

        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_chunk("Chu".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Q2h1".into()))
        );

        assert_eq!(
            c.finish_chunk()
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("".into()))
        );
    }*/

    #[test]
    fn eight_bit_encode() {
        let mut c = EightBitCodec::new();

        assert_eq!(
            c.encode_chunk("Hello, world!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, world!".into()))
        );

        assert_eq!(
            c.encode_chunk("Hello, мир!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, мир!".into()))
        );
    }

    #[test]
    fn binary_encode() {
        let mut c = BinaryCodec::new();

        assert_eq!(
            c.encode_chunk("Hello, world!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, world!".into()))
        );

        assert_eq!(
            c.encode_chunk("Hello, мир!".as_bytes())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, мир!".into()))
        );
    }
}
