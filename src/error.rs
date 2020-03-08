use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error")]
    Io {
        #[from]
        source: std::io::Error,
        // TODO: Add `backtrace` field once it's stabilized.
    },

    #[error("bad request")]
    BadRequest { source: bytecodec::Error },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    pub(crate) fn from_decode_error(e: bytecodec::Error) -> Self {
        Self::BadRequest { source: e }
    }
}
