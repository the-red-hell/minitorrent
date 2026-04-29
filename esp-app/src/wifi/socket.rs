use embassy_net::tcp::TcpSocket;

use crate::wifi::error::TcpError;

/// A connected TCP socket wrapper using `TcpError` for all operations.
///
/// This wrapper is necessary because embassy-net's `TcpSocket` uses different
/// error types for connect (`ConnectError`) and I/O (`Error`), but our
/// `TcpConnector` trait requires a single unified error type.
pub struct EspTcpSocket<'a>(TcpSocket<'a>);

impl<'a> EspTcpSocket<'a> {
    pub(crate) fn new(socket: TcpSocket<'a>) -> Self {
        Self(socket)
    }
}

impl<'a> embedded_io_async::ErrorType for EspTcpSocket<'a> {
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
