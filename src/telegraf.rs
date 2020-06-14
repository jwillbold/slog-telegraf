use std::{net, io};
use url::Url;
use crate::Error;
use std::io::Write;

/// Telegraf client
///
/// ```
/// use slog_telegraf::{TelegrafDrainBuilder, Client};
/// let mut client = Client::new("tcp://192.168.0.108:8094".into()).unwrap();
/// client.write("measurement,tag=value field=10i".as_bytes()).unwrap();
/// ```
pub struct Client {
    connection: Connection
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

enum Connection {
    Tcp(net::TcpStream),
    Udp(net::UdpSocket)
}

impl Connection {
    pub fn new(url: String) -> Result<Self, Error> {
        let url = Url::parse(&url)?;
        let addr = url.socket_addrs(|| None)?;

        match url.scheme() {
            "tcp" => Ok(Connection::Tcp(net::TcpStream::connect(&*addr)?)),
            "udp" => {
                // This will let the OS choose the ip+port
                let socket = net::UdpSocket::bind(&[net::SocketAddr::from(([0, 0, 0, 0], 0))][..])?;
                socket.connect(&*addr)?;
                socket.set_nonblocking(true)?;

                Ok(Connection::Udp(socket))
            },
            _ => Err(Error::Custom("Only 'tcp' is currently supported".to_string()))
        }
    }

    pub fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Connection::Tcp(tcp_stream) => tcp_stream.write(bytes),
            Connection::Udp(udp_socket) => udp_socket.send(bytes)
        }
    }
}