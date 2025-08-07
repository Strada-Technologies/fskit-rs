use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use prost::Message;
use tokio::io::AsyncWriteExt;
use tokio::io::Interest;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, RwLock};

use crate::error::Result;
use crate::pb::response::ResponseData;
use crate::pb::{Request, Response, ResponseTypeOne};
use crate::Error::SocketFailed;

const LOCALHOST_PORT: i32 = 35367;

#[derive(Clone, Debug)]
pub(super) struct Socket {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    stop_tx: Sender<()>,
}

impl Socket {
    pub(super) async fn start() -> Result<Self> {
        let (start_tx, mut start_rx) = mpsc::channel::<bool>(1);
        let (stop_tx, stop_rx) = mpsc::channel::<()>(1);

        let socket = Self {
            inner: Arc::new(RwLock::new(Inner { stop_tx })),
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

    //async fn spawn_loop<FS>(filesystem: FS, stop_rx: Receiver<()>) -> Result<()>
    // where
    //     FS: Filesystem + Send + 'static,
    async fn spawn_loop(&self, start_tx: &Sender<bool>, mut stop_rx: Receiver<()>) -> Result<()> {
        let addr = format!("{}:{LOCALHOST_PORT}", Ipv4Addr::LOCALHOST);

        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on {addr}");

        start_tx.send(true).await.unwrap();

        loop {
            select! {
                Ok((stream, peer)) = listener.accept() => {
                    println!("Accepted connection from {peer}");
                    tokio::spawn(Self::handle_client(stream));
                }
                _ = stop_rx.recv() => {
                    println!("Stop listening");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_client(mut stream: TcpStream) {
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

                                    // Build response
                                    let response = Response {
                                        request_id: request.request_id,
                                        response_data: Some(ResponseData::TypeOne(
                                            ResponseTypeOne {
                                                reply: "Pong".to_string(),
                                                success: true,
                                            },
                                        )),
                                    };

                                    // Encode response
                                    let mut out_buf = Vec::with_capacity(128);
                                    response.encode_length_delimited(&mut out_buf).unwrap();

                                    // Wait for write readiness
                                    stream.ready(Interest::WRITABLE).await.unwrap();
                                    match stream.try_write(&out_buf) {
                                        Ok(written) => println!("Wrote {} bytes", written),
                                        Err(e) => {
                                            eprintln!("Write error: {:?}", e);
                                            //return Err(e);
                                        }
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
