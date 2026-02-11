use core::net::{IpAddr, SocketAddr};
use embassy_net::{
    Stack,
    tcp::{TcpSocket, client::TcpClientState},
};
use embedded_nal_async::{Dns, TcpConnect};

mod network;
pub(crate) mod setup;

/// Network client that provides DNS resolution and TCP connections.
///
/// Buffer allocations:
/// - 2 concurrent TCP sockets
/// - 1.5KB TX buffer per socket = 3KB total
/// - 4KB RX buffer per socket = 8KB total
/// - Total: ~11KB of static memory
pub struct EspWifi {
    stack: Stack<'static>,
    // The state of the TCP client. We can have up to 2 concurrent connections.
    client_state: TcpClientState<2, 1536, 4096>,
    // The buffers for the TCP socket.
    // Note: This implementation can only have one active socket at a time.
    rx_buffer: [u8; 4096],
    tx_buffer: [u8; 1024],
}

impl EspWifi {
    pub fn new(stack: Stack<'static>) -> Self {
        Self {
            stack,
            client_state: TcpClientState::new(),
            rx_buffer: [0; 4096],
            tx_buffer: [0; 1024],
        }
    }
}

// Implement Dns trait for DNS resolution
impl Dns for EspWifi {
    type Error = embassy_net::dns::Error;

    async fn get_host_by_name(
        &self,
        host: &str,
        addr_type: embedded_nal_async::AddrType,
    ) -> Result<IpAddr, Self::Error> {
        if let embedded_nal_async::AddrType::IPv6 = addr_type {
            // Only IPv4 is supported in this example
            return Err(embassy_net::dns::Error::Failed);
        }

        let dns = embassy_net::dns::DnsSocket::new(self.stack);

        let addrs = dns.query(host, embassy_net::dns::DnsQueryType::A).await?;
        let addr = addrs.get(0).ok_or(embassy_net::dns::Error::Failed)?;
        match addr {
            embassy_net::IpAddress::Ipv4(ipv4_addr) => Ok(IpAddr::V4(*ipv4_addr)),
        }
    }

    async fn get_host_by_address(
        &self,
        _addr: IpAddr,
        _result: &mut [u8],
    ) -> Result<usize, Self::Error> {
        unreachable!("Reverse DNS lookup not used in this application");
    }
}

// Implement TcpConnect trait for TCP connections
impl TcpConnect for EspWifi {
    type Error = embassy_net::tcp::ConnectError;
    type Connection<'m>
        = embassy_net::tcp::TcpSocket<'m>
    where
        Self: 'm;

    async fn connect<'m>(
        &'m self,
        remote: SocketAddr,
    ) -> Result<Self::Connection<'m>, Self::Error> {
        let client = embassy_net::tcp::client::TcpClient::new(self.stack, &self.client_state);

        // Create the socket with the buffers owned by our struct
        let mut socket = TcpSocket::new(self.stack, &mut self.rx_buffer, &mut self.tx_buffer); // TODO: interior mutability for buffers or sth Idk

        socket.connect(remote).await?;
        Ok(socket)
    }
}
