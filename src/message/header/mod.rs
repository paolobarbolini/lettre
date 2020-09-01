/*!

## Headers widely used in email messages

*/

use std::fmt::{self, Display, Formatter};

// TODO: missing: Charset, DispositionParam, DispositionType
use headers::HeaderMapExt;
pub use headers::{
    ContentDisposition, ContentLocation, ContentType, Date, Error, Header, HeaderName, HeaderValue,
    UserAgent,
};
use headers_core::header::HeaderMap;

pub use self::{content::*, mailbox::*, special::*, textual::*};

mod content;
mod mailbox;
mod special;
mod textual;

#[derive(Debug, Clone)]
pub struct Headers(HeaderMap);

impl Headers {
    pub fn new() -> Self {
        Self(HeaderMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HeaderMap::with_capacity(capacity))
    }

    pub fn get<H>(&self) -> Option<H>
    where
        H: Header,
    {
        self.0.typed_get()
    }

    pub fn get_all<H>(&self) -> impl Iterator<Item = H> where H: Header {
        let mut values = self.0.get_all(H::name());
        H::decode(&mut values)
    }

    pub fn set<H>(&mut self, header: H)
    where
        H: Header,
    {
        self.0.typed_insert(header)
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for header in self.0.iter() {
            Display::fmt(&header, f)?;
        }
        Ok(())
    }
}
