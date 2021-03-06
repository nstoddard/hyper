//! Error and Result module.
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

use httparse;
#[cfg(feature = "openssl")]
use openssl::ssl::error::SslError;
use url;

use self::Error::*;


/// Result type often returned from methods that can have hyper `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;

/// A set of errors that can occur parsing HTTP streams.
#[derive(Debug)]
pub enum Error {
    /// An invalid `Method`, such as `GE,T`.
    Method,
    /// An invalid `RequestUri`, such as `exam ple.domain`.
    Uri(url::ParseError),
    /// An invalid `HttpVersion`, such as `HTP/1.1`
    Version,
    /// An invalid `Header`.
    Header,
    /// A message head is too large to be reasonable.
    TooLarge,
    /// An invalid `Status`, such as `1337 ELITE`.
    Status,
    /// An `io::Error` that occurred while trying to read or write to a network stream.
    Io(IoError),
    /// An error from the `openssl` library.
    #[cfg(feature = "openssl")]
    Ssl(SslError)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Method => "Invalid Method specified",
            Version => "Invalid HTTP version specified",
            Header => "Invalid Header provided",
            TooLarge => "Message head is too large",
            Status => "Invalid Status provided",
            Uri(ref e) => e.description(),
            Io(ref e) => e.description(),
            #[cfg(feature = "openssl")]
            Ssl(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Io(ref error) => Some(error),
            #[cfg(feature = "openssl")]
            Ssl(ref error) => Some(error),
            Uri(ref error) => Some(error),
            _ => None,
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Io(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Uri(err)
    }
}

#[cfg(feature = "openssl")]
impl From<SslError> for Error {
    fn from(err: SslError) -> Error {
        match err {
            SslError::StreamError(err) => Io(err),
            err => Ssl(err),
        }
    }
}

impl From<httparse::Error> for Error {
    fn from(err: httparse::Error) -> Error {
        match err {
            httparse::Error::HeaderName => Header,
            httparse::Error::HeaderValue => Header,
            httparse::Error::NewLine => Header,
            httparse::Error::Status => Status,
            httparse::Error::Token => Header,
            httparse::Error::TooManyHeaders => TooLarge,
            httparse::Error::Version => Version,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;
    use std::io;
    use httparse;
    use openssl::ssl::error::SslError;
    use url;
    use super::Error;
    use super::Error::*;

    #[test]
    fn test_cause() {
        let orig = io::Error::new(io::ErrorKind::Other, "other");
        let desc = orig.description().to_owned();
        let e = Io(orig);
        assert_eq!(e.cause().unwrap().description(), desc);
    }

    macro_rules! from {
        ($from:expr => $error:pat) => {
            match Error::from($from) {
                $error => (),
                _ => panic!("{:?}", $from)
            }
        }
    }

    #[test]
    fn test_from() {

        from!(io::Error::new(io::ErrorKind::Other, "other") => Io(..));
        from!(url::ParseError::EmptyHost => Uri(..));

        from!(SslError::StreamError(io::Error::new(io::ErrorKind::Other, "ssl")) => Io(..));
        from!(SslError::SslSessionClosed => Ssl(..));


        from!(httparse::Error::HeaderName => Header);
        from!(httparse::Error::HeaderName => Header);
        from!(httparse::Error::HeaderValue => Header);
        from!(httparse::Error::NewLine => Header);
        from!(httparse::Error::Status => Status);
        from!(httparse::Error::Token => Header);
        from!(httparse::Error::TooManyHeaders => TooLarge);
        from!(httparse::Error::Version => Version);
    }
}
