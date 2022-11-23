use futures::Future;
use mio::net;
use std::{
    io,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

pub struct ConnectFuture {}

impl Future for ConnectFuture {
    type Output = io::Result<net::TcpStream>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

pub async fn create_tcpstream_connection(address: SocketAddr) -> io::Result<ConnectFuture> {
    todo!()
}
