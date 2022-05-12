use std::net::IpAddr;
use thiserror::Error;
use tokio::net::TcpStream;

pub struct SQ {
    ip: IpAddr,
    socket: TcpStream,
}

impl SQ {
    pub fn new(ip: IpAddr) -> Result<Self, SQError> {
        let std_socket = std::net::TcpStream::connect((ip, 51325))?;
        std_socket.set_nonblocking(true)?;
        let socket = TcpStream::from_std(std_socket)?;

        Ok(SQ { ip, socket })
    }
}

#[derive(Error, Debug)]
pub enum SQError {
    #[error("Couldn't connect via TCP")]
    TcpConnectError(#[from] std::io::Error),
}
