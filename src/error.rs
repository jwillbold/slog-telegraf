use std::{io, error, fmt};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    UrlParsing(url::ParseError),
    Custom(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::UrlParsing(err) => err.fmt(f),
            Error::Custom(msg) => write!(f, "{}", msg)
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(err) => err.source(),
            Error::UrlParsing(_) |
            Error::Custom(_) => None
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParsing(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}