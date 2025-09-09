use std::net::Ipv4Addr;

use bytes::{Buf, BytesMut};
use prost::Message;
use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::Filesystem;
use crate::handler::Handler;
use crate::pb::{Request, Response};

pub type Result<T> = std::result::Result<T, Error>;

const LOCAL_PORT: i32 = 35367;

#[derive(Debug)]
pub(super) struct Socket {
    stop_tx: Sender<()>,
}

impl Socket {
    pub(super) async fn start<FS>(handler: Handler<FS>) -> Result<Self>
    where
        FS: Filesystem + Send + Sync + Clone + 'static,
    {
        let (start_tx, mut start_rx) = mpsc::channel::<bool>(1);
        let (stop_tx, stop_rx) = mpsc::channel::<()>(1);

        tokio::spawn(async move {
            if spawn_loop(&start_tx, stop_rx, handler).await.is_err() {
                start_tx.send(false).await.unwrap();
            }
        });

        if !start_rx.recv().await.unwrap() {
            return Err(Error::StartFailed);
        }

        Ok(Self { stop_tx })
    }

    pub(super) async fn stop(&self) {
        self.stop_tx.send(()).await.unwrap();
    }
}

async fn spawn_loop<FS>(
    start_tx: &Sender<bool>,
    mut stop_rx: Receiver<()>,
    handler: Handler<FS>,
) -> Result<()>
where
    FS: Filesystem + Send + Sync + Clone + 'static,
{
    let addr = format!("{}:{LOCAL_PORT}", Ipv4Addr::LOCALHOST);

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on {addr}");

    start_tx.send(true).await.unwrap();

    loop {
        select! {
            Ok((stream, peer)) = listener.accept() => {
                println!("Accepted connection from {peer}");
                let handler = handler.clone();
                tokio::spawn(async move {
                    handle_stream(stream, handler).await;
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

async fn handle_stream<FS>(mut stream: TcpStream, mut handler: Handler<FS>)
where
    FS: Filesystem + Send + Sync + Clone + 'static,
{
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

                                let content = handler.handle(request.content.unwrap()).await.ok();

                                let response = Response {
                                    request_id: request.id,
                                    content,
                                };

                                let mut out = Vec::with_capacity(4096);
                                response.encode_length_delimited(&mut out).unwrap();

                                if let Err(err) = stream.write_all(&out).await {
                                    eprintln!("Write error: {err}");
                                }
                            }
                            Err(err) => {
                                let s = err.to_string();
                                if !s.contains("failed to decode length prefix")
                                    && !s.contains("buffer underflow")
                                {
                                    eprintln!("Decode error: {err}");
                                }
                                break;
                            }
                        }
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(err) => eprintln!("Read error: {err}"),
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("socket failed to start")]
    StartFailed,
}
