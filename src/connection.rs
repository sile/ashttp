use crate::dispatcher::Dispatcher;
use crate::request::Req;
use crate::{Error, Result};
use async_std::io::{Read, Write};
use bytecodec::combinator::MaybeEos;
use bytecodec::{Decode, DecodeExt as _, Eos};
use httpcodec::{NoBodyDecoder, RequestDecoder};
use std::future::Future;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct ConnectionOptions {
    read_buf_size: usize,
    write_buf_size: usize,
    keepalive: bool,
}

#[derive(Debug)]
pub struct Connection<T> {
    stream: T,
    req_head_decoder: MaybeEos<RequestDecoder<NoBodyDecoder>>,
    dispatcher: Dispatcher,
    read_buf: ReadBuf,
    keepalive: bool,
    phase: Phase,
}

impl<T> Connection<T>
where
    T: Read + Write + Unpin,
{
    pub fn new(stream: T, dispatcher: Dispatcher) -> Self {
        // TODO: Use `RequestDecoder::with_options`
        let req_head_decoder = RequestDecoder::new(NoBodyDecoder);
        Self {
            stream,
            req_head_decoder: req_head_decoder.maybe_eos(),
            dispatcher,
            read_buf: ReadBuf::new(4096),
            keepalive: true,
            phase: Phase::ReadRequestHead,
        }
    }

    fn poll_once(&mut self, cx: &mut Context) -> Poll<Result<bool>> {
        match &self.phase {
            Phase::ReadRequestHead => {
                // TODO(error handling): invoke default handler and response the result, then close this connection
                self.read_request_head(cx)
            }
            Phase::DispatchRequest(req) => {
                todo!();
            }
        }
    }

    fn read_request_head(&mut self, cx: &mut Context) -> Poll<Result<bool>> {
        if self.read_buf.read(&mut self.stream, cx)?.is_pending() {
            return Poll::Pending;
        }
        if let Some(head) = self.read_buf.decode(&mut self.req_head_decoder)? {
            let req = Req::new(head)?;
            self.phase = Phase::DispatchRequest(req);
        }
        Poll::Ready(Ok(false))
    }
}

#[derive(Debug)]
struct ReadBuf {
    buf: Vec<u8>,
    head: usize,
    tail: usize,
    eos: bool,
}

impl ReadBuf {
    fn new(buf_size: usize) -> Self {
        Self {
            buf: vec![0; buf_size],
            head: 0,
            tail: 0,
            eos: false,
        }
    }

    fn decode<T: Decode>(&mut self, decoder: &mut T) -> Result<Option<T::Item>> {
        match decoder.decode(&self.buf[self.head..self.tail], Eos::new(self.eos)) {
            Err(e) => {
                if *e.kind() == bytecodec::ErrorKind::UnexpectedEos {
                    let kind = std::io::ErrorKind::UnexpectedEof;
                    Err(std::io::Error::new(kind, e).into())
                } else {
                    Err(Error::BadRequest(anyhow::Error::new(e)))
                }
            }
            Ok(size) => {
                self.head += size;
                if decoder.is_idle() {
                    decoder
                        .finish_decoding()
                        .map(Some)
                        .map_err(anyhow::Error::new)
                        .map_err(Error::BadRequest)
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn read(&mut self, stream: &mut (impl Read + Unpin), cx: &mut Context) -> Poll<Result<()>> {
        if self.eos {
            return Poll::Ready(Ok(()));
        }

        if self.head == self.tail && self.head != 0 {
            self.head = 0;
            self.tail = 0;
        }

        match Pin::new(stream).poll_read(cx, &mut self.buf[self.tail..]) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Ready(Ok(size)) => {
                self.eos = size == 0;
                self.tail += size;
                Poll::Ready(Ok(()))
            }
        }
    }
}

impl<T> Future for Connection<T>
where
    T: Read + Write + Unpin,
{
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            match self.poll_once(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Ready(Ok(true)) => return Poll::Ready(Ok(())),
                Poll::Ready(Ok(false)) => {}
            }
        }
    }
}

#[derive(Debug)]
enum Phase {
    ReadRequestHead,
    DispatchRequest(Req<()>),
}
