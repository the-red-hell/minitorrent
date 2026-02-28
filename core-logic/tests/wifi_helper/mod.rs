use std::{
    fmt::Display,
    net::{IpAddr, SocketAddr, SocketAddrV4},
};

use core_logic::TcpConnector;
use embedded_io::ErrorType;
use embedded_nal_async::{AddrType, Dns};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

// pub const IP_ADDRESS: &Ipv4Addr = &std::net::Ipv4Addr::new(192, 168, 1, 42);

#[derive(Debug)]
pub struct WifiHelper;

/// Wrapper around tokio's `TcpStream` that implements `embedded_io_async` traits.
///
/// Unlike embassy's `TcpSocket`, tokio manages its own internal buffers,
/// so we don't need to store the external buffers passed to `connect()`.
#[derive(Debug)]
pub struct TcpConnectionDuple(TcpStream);

impl TcpConnector for WifiHelper {
    type Error = WifiError;
    type Connection<'a> = TcpConnectionDuple;

    /// Connect to a remote address.
    ///
    /// # Note on buffers
    ///
    /// The `rx_buffer` and `tx_buffer` parameters are **ignored** in this
    /// test implementation. Tokio's `TcpStream` manages its own internal
    /// buffers, unlike embedded implementations (embassy) which require
    /// caller-provided buffers.
    ///
    /// This is intentional - the trait is designed for embedded systems,
    /// but for testing we use tokio which handles buffering transparently.
    async fn connect<'a>(
        &'a self,
        remote: SocketAddrV4,
        _rx_buffer: &'a mut [u8], // tokio manages its own buffers
        _tx_buffer: &'a mut [u8], // tokio manages its own buffers
    ) -> Result<Self::Connection<'a>, Self::Error> {
        let stream = TcpStream::connect(remote).await.map_err(WifiError::from)?;
        Ok(TcpConnectionDuple(stream))
    }
}

impl Dns for WifiHelper {
    type Error = WifiError;

    async fn get_host_by_name(
        &self,
        hostname: &str,
        addr_type: AddrType,
    ) -> Result<IpAddr, Self::Error> {
        // Parse URL to extract hostname properly
        // Use tokio's DNS resolution - append port to satisfy lookup_host requirements
        let addrs: Vec<SocketAddrV4> = tokio::net::lookup_host(format!("{}:0", hostname))
            .await
            .map_err(WifiError::from)?
            .filter_map(|addr| match addr {
                SocketAddr::V4(v4) => Some(v4),
                SocketAddr::V6(_) => None,
            })
            .collect();

        // Filter by requested address type
        for addr in addrs {
            let ip = addr.ip();
            match addr_type {
                AddrType::IPv4 => return Ok(IpAddr::V4(*ip)),
                _ => continue,
            }
        }

        Err(WifiError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No matching address found for host",
        )))
    }

    async fn get_host_by_address(
        &self,
        _addr: std::net::IpAddr,
        _result: &mut [u8],
    ) -> Result<usize, Self::Error> {
        todo!("get_host_by_address is not implemented in DnsDuple")
    }
}

impl embedded_io_async::Read for TcpConnectionDuple {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(buf).await.map_err(WifiError::from)
    }
}

impl embedded_io_async::Write for TcpConnectionDuple {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf).await.map_err(WifiError::from)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().await.map_err(WifiError::from)
    }
}

impl ErrorType for TcpConnectionDuple {
    type Error = WifiError;
}
#[derive(Debug)]
pub struct WifiError(pub std::io::Error);

impl Display for WifiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WifiError: {}", self.0)
    }
}
impl std::error::Error for WifiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl embedded_io::Error for WifiError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl defmt::Format for WifiError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "WifiError: {:?}", defmt::Debug2Format(&self.0));
    }
}

impl From<std::io::Error> for WifiError {
    fn from(err: std::io::Error) -> Self {
        WifiError(err)
    }
}
