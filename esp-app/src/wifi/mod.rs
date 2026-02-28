use core::{
    fmt::Display,
    net::{IpAddr, SocketAddrV4},
};
use core_logic::TcpConnector;
use embassy_net::{Stack, tcp::TcpSocket};
use embedded_nal_async::Dns;

mod network;
pub(crate) mod setup;

// ============================================================================
// Error Types
// ============================================================================

/// Unified TCP error type that wraps both connection and I/O errors.
///
/// Embassy-net uses different error types for connection (`ConnectError`)
/// and I/O operations (`Error`). This wrapper unifies them for the
/// `TcpConnector` trait which requires a single error type.
#[derive(Debug)]
pub enum TcpError {
    /// Error during connection establishment (DNS, timeout, refused, etc.)
    Connect(embassy_net::tcp::ConnectError),
    /// Error during read/write operations
    Io(embassy_net::tcp::Error),
}

impl Display for TcpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TcpError::Connect(e) => write!(f, "TCP connection error: {}", e),
            TcpError::Io(e) => write!(f, "TCP I/O error: {}", e),
        }
    }
}

impl ::core::error::Error for TcpError {}

impl From<embassy_net::tcp::ConnectError> for TcpError {
    fn from(err: embassy_net::tcp::ConnectError) -> Self {
        TcpError::Connect(err)
    }
}

impl From<embassy_net::tcp::Error> for TcpError {
    fn from(err: embassy_net::tcp::Error) -> Self {
        TcpError::Io(err)
    }
}

impl embedded_io::Error for TcpError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            TcpError::Connect(_) => embedded_io::ErrorKind::ConnectionRefused,
            TcpError::Io(e) => e.kind(),
        }
    }
}

// ============================================================================
// EspWifi - Network Client
// ============================================================================

/// WiFi network client for ESP32 that provides DNS resolution and TCP connections.
///
/// # Design
///
/// This struct is intentionally lightweight - it only holds a reference to the
/// embassy-net `Stack`. **It does not own TCP socket buffers.** Instead, buffers
/// are provided by the caller (e.g., `BitTorrenter`) when establishing connections.
///
/// This design avoids:
/// - Interior mutability (RefCell/Mutex) for buffer access
/// - Runtime borrow checking overhead
/// - Ownership complexity in embedded async code
///
/// # Example
///
/// ```ignore
/// let wifi = EspWifi::new(stack);
///
/// // Buffers are provided externally
/// let mut rx = [0u8; 4096];
/// let mut tx = [0u8; 1024];
/// let socket = wifi.connect(addr, &mut rx, &mut tx).await?;
/// ```
pub struct EspWifi {
    /// The embassy-net network stack.
    ///
    /// Handles IP routing, TCP state machines, and the WiFi driver interface.
    stack: Stack<'static>,
}

impl EspWifi {
    /// Create a new WiFi client wrapping the given network stack.
    ///
    /// The stack should already be initialized and connected to a network.
    pub fn new(stack: Stack<'static>) -> Self {
        Self { stack }
    }

    /// Get access to the underlying network stack.
    ///
    /// Useful for advanced operations not exposed by this wrapper.
    pub fn stack(&self) -> Stack<'static> {
        self.stack
    }
}

// ============================================================================
// DNS Resolution
// ============================================================================

impl Dns for EspWifi {
    type Error = embassy_net::dns::Error;

    /// Resolve a hostname to an IP address.
    ///
    /// Only IPv4 is supported in this implementation.
    async fn get_host_by_name(
        &self,
        host: &str,
        addr_type: embedded_nal_async::AddrType,
    ) -> Result<IpAddr, Self::Error> {
        if let embedded_nal_async::AddrType::IPv6 = addr_type {
            return Err(embassy_net::dns::Error::Failed);
        }

        let dns = embassy_net::dns::DnsSocket::new(self.stack);
        let addrs = dns.query(host, embassy_net::dns::DnsQueryType::A).await?;
        let addr = addrs.first().ok_or(embassy_net::dns::Error::Failed)?;

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

// ============================================================================
// TCP Connections (caller provides buffers)
// ============================================================================

/// A connected TCP socket wrapper that uses `TcpError` for all operations.
///
/// This wrapper is necessary because embassy-net's `TcpSocket` uses different
/// error types for connect (`ConnectError`) and I/O (`Error`), but our
/// `TcpConnector` trait requires a single unified error type.
pub struct EspTcpSocket<'a>(TcpSocket<'a>);

impl<'a> embedded_io::ErrorType for EspTcpSocket<'a> {
    type Error = TcpError;
}

impl<'a> embedded_io_async::Read for EspTcpSocket<'a> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(buf).await.map_err(TcpError::from)
    }
}

impl<'a> embedded_io_async::Write for EspTcpSocket<'a> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf).await.map_err(TcpError::from)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().await.map_err(TcpError::from)
    }
}

impl TcpConnector for EspWifi {
    type Error = TcpError;
    type Connection<'a> = EspTcpSocket<'a>;

    /// Establish a TCP connection to a remote address.
    ///
    /// # Arguments
    ///
    /// * `remote` - The IP address and port to connect to
    /// * `rx_buffer` - Buffer for incoming data (caller-owned)
    /// * `tx_buffer` - Buffer for outgoing data (caller-owned)
    ///
    /// # Returns
    ///
    /// A connected `EspTcpSocket` that borrows the provided buffers.
    /// The socket is dropped when the connection is closed.
    ///
    /// # Buffer Sizing
    ///
    /// - `rx_buffer`: Larger = better throughput, more memory. 4KB is typical.
    /// - `tx_buffer`: Affects how much data can be in-flight. 1KB is typical.
    async fn connect<'a>(
        &'a self,
        remote: SocketAddrV4,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> Result<Self::Connection<'a>, Self::Error> {
        let mut socket = TcpSocket::new(self.stack, rx_buffer, tx_buffer);
        socket.connect(remote).await?;
        Ok(EspTcpSocket(socket))
    }
}
