use std::{
    fmt::Display,
    net::{IpAddr, SocketAddr},
};

use embedded_io::ErrorType;
use embedded_nal_async::{AddrType, Dns, TcpConnect};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

// pub const IP_ADDRESS: &Ipv4Addr = &std::net::Ipv4Addr::new(192, 168, 1, 42);

#[derive(Debug)]
pub struct WifiHelper;

#[derive(Debug)]
pub struct TcpConnectionDuple(TcpStream);

impl TcpConnect for WifiHelper {
    type Error = WifiError;
    type Connection<'m> = TcpConnectionDuple;

    async fn connect<'m>(
        &'m self,
        remote: SocketAddr,
    ) -> Result<Self::Connection<'m>, Self::Error> {
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
        let addrs: Vec<SocketAddr> = tokio::net::lookup_host(format!("{}:0", hostname))
            .await
            .map_err(WifiError::from)?
            .collect();

        // Filter by requested address type
        for addr in addrs {
            let ip = addr.ip();
            match addr_type {
                AddrType::IPv4 if ip.is_ipv4() => return Ok(ip),
                AddrType::IPv6 if ip.is_ipv6() => return Ok(ip),
                AddrType::Either => return Ok(ip),
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
