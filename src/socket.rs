use std::net::Ipv4Addr;

use bytes::{Buf, BytesMut};
use log::{debug, error, info, warn};
use prost::Message;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};

use crate::Filesystem;
use crate::handler::Handler;
use crate::pb::{Request, Response, response};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(super) struct Socket {
    stop_tx: Sender<()>,
}

impl Socket {
    pub(super) async fn start<FS>(handler: Handler<FS>, server_port: u16) -> Result<Self>
    where
        FS: Filesystem + Send + Sync + Clone + 'static,
    {
        let (start_tx, mut start_rx) = mpsc::channel::<bool>(1);
        let (stop_tx, stop_rx) = mpsc::channel::<()>(1);

        tokio::spawn(async move {
            if spawn_loop(&start_tx, stop_rx, handler, server_port)
                .await
                .is_err()
            {
                let _ = start_tx.send(false).await;
            }
        });

        if !start_rx.recv().await.unwrap_or(false) {
            return Err(Error::StartFailed);
        }

        Ok(Self { stop_tx })
    }

    pub(super) async fn stop(&self) {
        let _ = self.stop_tx.send(()).await;
    }
}

async fn spawn_loop<FS>(
    start_tx: &Sender<bool>,
    mut stop_rx: Receiver<()>,
    handler: Handler<FS>,
    server_port: u16,
) -> Result<()>
where
    FS: Filesystem + Send + Sync + Clone + 'static,
{
    let addr = format!("{}:{}", Ipv4Addr::LOCALHOST, server_port);

    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {addr}");

    let _ = start_tx.send(true).await;

    let (shutdown_tx, _) = broadcast::channel::<()>(2);

    loop {
        select! {
            _ = stop_rx.recv() => {
                info!("stop listening");
                let _ = shutdown_tx.send(());
                break;
            }

            Ok((stream, peer)) = listener.accept() => {
                info!("accepted connection from {peer}");
                let handler = handler.clone();
                let shutdown_rx = shutdown_tx.subscribe();
                tokio::spawn(async move {
                    if let Err(err) = handle_stream(stream, handler, shutdown_rx).await {
                        error!("{err}");
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_stream<FS>(
    stream: TcpStream,
    handler: Handler<FS>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()>
where
    FS: Filesystem + Send + Sync + Clone + 'static,
{
    // Split into independent read and write halves so multiple in-flight
    // handler tasks can produce responses concurrently, with a single
    // writer task serializing the bytes onto the wire.
    let (read_half, mut write_half) = stream.into_split();
    let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    let writer = tokio::spawn(async move {
        while let Some(out) = resp_rx.recv().await {
            if let Err(err) = write_half.write_all(&out).await {
                error!("write error: {err}");
                break;
            }
        }
        let _ = write_half.shutdown().await;
    });

    let mut buf = BytesMut::with_capacity(4096);
    loop {
        select! {
            _ = shutdown_rx.recv() => {
                info!("connection closed by shutdown");
                drop(resp_tx);
                let _ = writer.await;
                return Ok(());
            }

            r = read_half.readable() => {
                r?;
                match read_half.try_read_buf(&mut buf) {
                    Ok(0) => {
                        info!("connection closed by peer");
                        drop(resp_tx);
                        let _ = writer.await;
                        return Ok(());
                    }
                    Ok(_) => {
                        while buf.has_remaining() {
                            let mut frozen = buf.clone().freeze();
                            match Request::decode_length_delimited(&mut frozen) {
                                Ok(request) => {
                                    debug!("received message: {request:?}");
                                    buf.advance(buf.len() - frozen.remaining());

                                    let request_id = request.id;
                                    let content_in = request.content;
                                    let mut handler = handler.clone();
                                    let resp_tx = resp_tx.clone();

                                    tokio::spawn(async move {
                                        let content = match content_in {
                                            Some(content) => match handler.handle(content).await {
                                                Ok(content) => Some(content),
                                                Err(err) => {
                                                    error!("handler error: {err}");
                                                    None
                                                }
                                            },
                                            None => {
                                                warn!("received request without content: {request_id}");
                                                Some(response::Content::PosixError(libc::EINVAL))
                                            }
                                        };

                                        let response = Response { request_id, content };

                                        let mut out = Vec::with_capacity(4096);
                                        if let Err(err) = response.encode_length_delimited(&mut out) {
                                            error!("encode error: {err}");
                                            return;
                                        }

                                        let _ = resp_tx.send(out);
                                    });
                                }
                                Err(err) => {
                                    let s = err.to_string();
                                    if !s.contains("failed to decode length prefix")
                                        && !s.contains("buffer underflow")
                                    {
                                        error!("decode error: {err}");
                                        drop(resp_tx);
                                        let _ = writer.await;
                                        return Err(err.into());
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(err) => {
                        error!("read error: {err}");
                        drop(resp_tx);
                        let _ = writer.await;
                        return Err(err.into());
                    }
                }
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    DecodeError(#[from] prost::DecodeError),

    #[error("socket failed to start")]
    StartFailed,
}
