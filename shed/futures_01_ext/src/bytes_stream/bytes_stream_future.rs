/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use bytes_old::Bytes;
use futures::try_ready;
use futures::Async;
use futures::Future;
use futures::Poll;
use futures::Stream;
use tokio_io::codec::Decoder;

use super::BytesStream;

/// A future that yields a single decoded item from the Bytes of the provided
/// BytesStream (if any) and the remaining BytesStream.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct BytesStreamFuture<S, Dec> {
    inner: Option<BytesStreamFutureInner<S, Dec>>,
}

impl<S, Dec> BytesStreamFuture<S, Dec>
where
    S: Stream<Item = Bytes>,
    Dec: Decoder,
    Dec::Error: From<S::Error>,
{
    pub(crate) fn new(bs: BytesStream<S>, decoder: Dec) -> Self {
        let is_readable = !bs.bytes.is_empty() || bs.stream_done;

        BytesStreamFuture {
            inner: Some(BytesStreamFutureInner {
                bs,
                decoder,
                is_readable,
            }),
        }
    }
}

impl<S, Dec> Future for BytesStreamFuture<S, Dec>
where
    S: Stream<Item = Bytes>,
    Dec: Decoder,
    Dec::Error: From<S::Error>,
{
    type Item = (Option<Dec::Item>, BytesStream<S>);
    type Error = (Dec::Error, BytesStream<S>);

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut inner = self
            .inner
            .take()
            .expect("calling poll after future completed");
        match inner.poll() {
            Ok(Async::NotReady) => {
                self.inner = Some(inner);
                Ok(Async::NotReady)
            }
            Ok(Async::Ready(frame)) => Ok(Async::Ready((frame, inner.bs))),
            Err(frame) => Err((frame, inner.bs)),
        }
    }
}

struct BytesStreamFutureInner<S, Dec> {
    bs: BytesStream<S>,
    decoder: Dec,
    is_readable: bool,
}

impl<S, Dec> BytesStreamFutureInner<S, Dec>
where
    S: Stream<Item = Bytes>,
    Dec: Decoder,
    Dec::Error: From<S::Error>,
{
    fn poll(&mut self) -> Poll<Option<Dec::Item>, Dec::Error> {
        loop {
            if self.is_readable {
                if self.bs.stream_done {
                    return Ok(Async::Ready(self.decoder.decode_eof(&mut self.bs.bytes)?));
                }

                if let Some(frame) = self.decoder.decode(&mut self.bs.bytes)? {
                    return Ok(Async::Ready(Some(frame)));
                }

                self.is_readable = false;
            }

            try_ready!(self.bs.poll_buffer());
            self.is_readable = true;
        }
    }
}
