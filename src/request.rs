use crate::{Error, Result};
use anyhow::anyhow;
use httpcodec::Request;
use url::Url;

#[derive(Debug)]
pub struct Req<T> {
    inner: Request<T>,
    url: Url,
}

impl<T> Req<T> {
    pub(crate) fn new(inner: Request<T>) -> Result<Self> {
        if !inner.request_target().as_str().starts_with('/') {
            return Err(Error::BadRequest(anyhow!(
                "Unsupported request target: {:?}",
                inner.request_target().as_str()
            )));
        }

        let host = inner
            .header()
            .get_field("Host")
            .map(|host| format!("http://{}/", host))
            .ok_or_else(|| Error::BadRequest(anyhow!("Missing HOST header")))?;

        let base_url = Url::parse(&host)
            .map_err(anyhow::Error::new)
            .map_err(Error::BadRequest)?;

        let url = Url::options()
            .base_url(Some(&base_url))
            .parse(inner.request_target().as_str())
            .map_err(anyhow::Error::new)
            .map_err(Error::BadRequest)?;

        Ok(Self { inner, url })
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Req<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
