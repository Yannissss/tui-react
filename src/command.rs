#![allow(unused)]
use futures::{Future, Stream};
use std::pin::Pin;

pub enum Command<M> {
    None,
    Instant(M),
    Single(Pin<Box<dyn Future<Output = M> + Send + 'static>>),
    Stream(Pin<Box<dyn Stream<Item = M> + Send + 'static>>),
}

impl<M> Command<M>
where
    M: Send + 'static,
{
    #[inline(always)]
    pub fn none() -> Self {
        Self::None
    }

    #[inline(always)]
    pub fn instant(message: M) -> Self {
        Self::Instant(message)
    }

    #[inline(always)]
    pub fn single<F>(future: F) -> Self
    where
        F: Future<Output = M> + Send + 'static,
    {
        Self::Single(Box::pin(future))
    }

    #[inline(always)]
    pub fn stream<S>(stream: S) -> Self
    where
        S: Stream<Item = M> + Send + 'static,
    {
        Self::Stream(Box::pin(stream))
    }
}

impl<M> Default for Command<M> {
    #[inline(always)]
    fn default() -> Self {
        Self::None
    }
}
