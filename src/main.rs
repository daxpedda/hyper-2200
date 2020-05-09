use futures::ready;
use hyper::{
    server::{
        accept::Accept,
        conn::{AddrIncoming, AddrStream},
    },
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use std::{
    io::Error,
    net::SocketAddr,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

#[tokio::main]
async fn main() {
    let incoming = Acceptor::new(
        AddrIncoming::bind(&SocketAddr::from_str("127.0.0.1:8000").unwrap()).unwrap(),
    );

    let make_svc = make_service_fn(|_| async {
        Ok::<_, Error>(service_fn(|_| async {
            Ok::<_, Error>(Response::new(Body::from("Hello World")))
        }))
    });

    Server::builder(incoming).serve(make_svc).await.unwrap();
}

struct Acceptor {
    incoming: AddrIncoming,
}

impl Acceptor {
    fn new(incoming: AddrIncoming) -> Self {
        Self { incoming }
    }
}

impl Accept for Acceptor {
    type Conn = AddrStream;
    type Error = Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let pin = self.get_mut();
        match ready!(Pin::new(&mut pin.incoming).poll_accept(cx)) {
            Some(Ok(mut sock)) => {
                let mut byte = [0];
                // `poll_peek` doesn't arrange the task to be awoken again
                // therefore on `Poll::Pending` this will never awaken again
                ready!(sock.poll_peek(cx, &mut byte))?;
                Poll::Ready(Some(Ok(sock)))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}
