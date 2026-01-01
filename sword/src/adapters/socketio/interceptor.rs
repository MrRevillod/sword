use crate::{adapters::socketio::extract::SocketContext, interceptor::Interceptor};
use socketioxide::adapter::{Adapter as SocketIoAdapter, LocalAdapter};
use std::fmt::Display;

#[allow(async_fn_in_trait)]
pub trait OnConnect<A = LocalAdapter>: Interceptor
where
    A: SocketIoAdapter,
{
    type Error: Display;

    async fn on_connect(&self, socket: SocketContext<A>) -> Result<(), Self::Error>;
}

#[derive(Copy, Clone)]
pub enum SocketIoParser {
    Common,
    MsgPack,
}
