use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use prost::Message;
use tokio::io::Interest;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{RwLock, mpsc};

use crate::Error::SocketFailed;
use crate::Filesystem;
use crate::error::Result;
use crate::handler::Handler;
use crate::pb::{Request, Response};

const LOCALHOST_PORT: i32 = 35367;

#[derive(Debug)]
pub(super) struct Socket<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    inner: Arc<RwLock<Inner<FS>>>,
}

#[derive(Debug)]
struct Inner<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    handler: Handler<FS>,
    stop_tx: Sender<()>,
}

impl<FS> Socket<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    pub(super) async fn start(handler: Handler<FS>) -> Result<Self> {
        let (start_tx, mut start_rx) = mpsc::channel::<bool>(1);
        let (stop_tx, stop_rx) = mpsc::channel::<()>(1);

        let socket = Self {
            inner: Arc::new(RwLock::new(Inner { handler, stop_tx })),
        };

        let socket2 = socket.clone();
        tokio::spawn(async move {
            if socket2.spawn_loop(&start_tx, stop_rx).await.is_err() {
                start_tx.send(false).await.unwrap();
            }
        });

        if !start_rx.recv().await.unwrap() {
            return Err(SocketFailed);
        }

        Ok(socket)
    }

    pub(super) async fn stop(&self) {
        let inner = &mut self.inner.write().await;
        inner.stop_tx.send(()).await.unwrap()
    }

    async fn spawn_loop(&self, start_tx: &Sender<bool>, mut stop_rx: Receiver<()>) -> Result<()> {
        let addr = format!("{}:{LOCALHOST_PORT}", Ipv4Addr::LOCALHOST);

        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on {addr}");

        start_tx.send(true).await.unwrap();

        loop {
            select! {
                Ok((stream, peer)) = listener.accept() => {
                    println!("Accepted connection from {peer}");
                    let this  = self.clone();
                    tokio::spawn(async move {
                        this.handle_stream(stream).await;
                    });
                }
                _ = stop_rx.recv() => {
                    println!("Stop listening");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_stream(&self, stream: TcpStream) {
        let mut buf = BytesMut::with_capacity(4096);
        loop {
            if stream.ready(Interest::READABLE).await.is_ok() {
                match stream.try_read_buf(&mut buf) {
                    Ok(0) => {
                        println!("Connection closed: {stream:?}");
                        break;
                    }
                    Ok(_) => {
                        while buf.has_remaining() {
                            let mut frozen = buf.clone().freeze();
                            match Request::decode_length_delimited(&mut frozen) {
                                Ok(request) => {
                                    println!("Received message: {request:?}");
                                    buf.advance(buf.len() - frozen.remaining());

                                    let content = {
                                        let handler = &mut self.inner.write().await.handler;
                                        handler.handle(request.content.unwrap()).await.ok()
                                    };

                                    let response = Response {
                                        request_id: request.id,
                                        content,
                                    };

                                    let mut out_buf = Vec::with_capacity(4096);
                                    response.encode_length_delimited(&mut out_buf).unwrap();

                                    stream.ready(Interest::WRITABLE).await.unwrap();
                                    if let Err(e) = stream.try_write(&out_buf) {
                                        eprintln!("Write error: {e:?}");
                                    }
                                }
                                Err(e)
                                    if e.to_string().contains("failed to decode length prefix") =>
                                {
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("Decode error: {e:?}");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => eprintln!("Read error: {e:?}"),
                }
            }
        }
    }
}

impl<FS> Clone for Socket<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
