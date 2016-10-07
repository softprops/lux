use super::hyper;
use super::serde_json;
use super::url;

#[derive(Debug)]
pub enum Error {
    Http,
    Parse,
    Url,
}

impl From<url::ParseError> for Error {
    fn from(_: url::ParseError) -> Error {
        Error::Url
    }
}

impl From<hyper::Error> for Error {
    fn from(_: hyper::Error) -> Error {
        Error::Http
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Error::Parse
    }
}
