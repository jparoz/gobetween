use bytes::BytesMut;
use std::net::IpAddr;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::{broadcast, mpsc};

pub mod id; // pub?
pub mod message;

use message::Message;

pub struct SQ {
    ip: IpAddr,
    // socket: TcpStream,

    // This is a clone of the sender moved to the master callback.
    // Use sq.subscribe() to receive SQ state update messages.
    tx: broadcast::Sender<Message>,

    // This is the sender to which we send SQ state update messages.
    update_tx: mpsc::Sender<Message>,
}

impl SQ {
    pub fn new(ip: IpAddr) -> Result<Self, SQError> {
        let (tx, _rx) = broadcast::channel(128); // @TestMe: is this the right capacity?
        let (update_tx, mut update_rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
            mpsc::channel(4);
        let cloned_tx = tx.clone();

        tokio::spawn(async move {
            // @Fixme: shouldn't unwrap
            let mut socket = TcpStream::connect((ip, 51325)).await.unwrap();
            let mut buf = BytesMut::new();

            loop {
                select! {
                    bytes_read = socket.read_buf(&mut buf) => {
                        // @Fixme: shouldn't unwrap
                        let _bytes_read = bytes_read.unwrap();
                        if let Some(nrpn) = message::Nrpn::from_buf(&mut buf) {
                            // @Fixme: shouldn't unwrap
                            cloned_tx.send(Message::from_nrpn(nrpn)).unwrap();
                        }
                    }
                    msg = update_rx.recv() => {
                        let msg = msg.unwrap(); // @Fixme: shouldn't unwrap, maybe pattern match?
                        socket.write(&msg.to_nrpn().to_bytes()).await.unwrap(); // @Fixme: shouldn't unwrap
                    }
                }
            }
        });

        Ok(SQ { ip, tx, update_tx })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.tx.subscribe()
    }

    pub async fn update(&mut self, msg: Message) -> Result<(), SQError> {
        self.update_tx.send(msg).await?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SQError {
    #[error("TCP error: {0}")]
    TcpError(#[from] std::io::Error),

    #[error("Couldn't send data via tokio MPSC channel: {0}")]
    MpscSendError(#[from] tokio::sync::mpsc::error::SendError<Message>),
}
