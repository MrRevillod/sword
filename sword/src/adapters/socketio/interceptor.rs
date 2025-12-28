use crate::interceptor::Interceptor;
use socketioxide::{
    adapter::{Adapter as SocketIoAdapter, LocalAdapter},
    extract::SocketRef,
    handler::FromConnectParts,
};
use std::fmt::Display;

#[allow(async_fn_in_trait)]
pub trait OnConnect<A = LocalAdapter>: Interceptor
where
    A: SocketIoAdapter,
{
    type Error: Display;

    async fn on_connect(&self, socket: SocketRef<A>) -> Result<(), Self::Error>;
}

#[allow(async_fn_in_trait)]
pub trait OnConnectWith<E, A = LocalAdapter>: Interceptor
where
    A: SocketIoAdapter,
    E: FromConnectParts<A>,
{
    type Error: Display;

    async fn on_connect_with(
        &self,
        socket: SocketRef<A>,
        extractor: E,
    ) -> Result<(), Self::Error>;
}
