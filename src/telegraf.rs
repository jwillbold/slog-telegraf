use std::{net, io};
use url::Url;
use crate::Error;
use std::io::Write;

pub enum Connection {
    Tcp(net::TcpStream)
}

impl Connection {
    pub fn new(url: String) -> Result<Self, Error> {
        let url = Url::parse(&url)?;
        let addr = url.socket_addrs(|| None)?;

        match url.scheme() {
            "tcp" => Ok(Connection::Tcp(net::TcpStream::connect(&*addr)?)),
            _ => Err(Error::Custom("Only 'tcp' is currently supported".to_string()))
        }
    }

    pub fn write(&mut self, bytes:&[u8]) -> io::Result<usize> {
        match self {
            Connection::Tcp(tcp_stream) => tcp_stream.write(bytes)
        }
    }
}

pub struct Client {
    pub connection: Connection
}

impl Client {
    pub fn new(url: String) -> Result<Self, Error> {
        Ok(Client{
            connection: Connection::new(url)?
        })
    }

    pub fn write(&mut self, bytes:&[u8]) -> io::Result<()> {
        self.connection.write(bytes).map(|_| ())
    }
}