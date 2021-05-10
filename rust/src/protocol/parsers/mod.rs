use nom::error::{ErrorKind, ParseError};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

mod combinators;
mod coordinate;
mod encoding_algorithm;
mod help_topic;
mod hex_color;
mod request;
mod response;

pub(super) use request::parse as parse_request;

pub struct Error(anyhow::Error);

impl Error {
    fn msg<M>(msg: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Self(anyhow::Error::msg(msg))
    }
}

impl ParseError<&str> for Error {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        Self(anyhow::Error::msg(format!(
            "nom {} error while parsing '{}'",
            kind.description(),
            input
        )))
    }

    fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
        Self(anyhow::Error::from(other.0).context(format!(
            "nom {} error while parsing '{}'",
            kind.description(),
            input
        )))
    }
}

impl Deref for Error {
    type Target = anyhow::Error;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<anyhow::Error> for Error {
    fn into(self) -> anyhow::Error {
        self.0
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", *self))
    }
}
