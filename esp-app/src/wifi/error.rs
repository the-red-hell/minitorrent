use core::fmt::Display;

/// Unified TCP error type that wraps both connection and I/O errors.
///
/// Embassy-net uses different error types for connection (`ConnectError`)
/// and I/O operations (`Error`). This wrapper unifies them for the
/// `TcpConnector` trait which requires a single error type.
#[derive(Debug, defmt::Format)]
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

impl embedded_io_async::Error for TcpError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            TcpError::Connect(_) => embedded_io_async::ErrorKind::ConnectionRefused,
            TcpError::Io(e) => e.kind(),
        }
    }
}
