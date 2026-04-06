use ::core::net::SocketAddrV4;

use embedded_io_async::{Read, Write};

/// A trait for establishing TCP connections where the **caller provides buffers**.
///
/// Unlike `embedded_nal_async::TcpConnect`, this trait accepts mutable buffer
/// references as parameters. It avoids interior mutability (RefCell,
/// Mutex) in the network implementation, which better for embedded single-threaded
/// applications where you don't have too much resources.
///
/// # Buffer Lifetimes
///
/// The returned `Connection` borrows from the buffers, so the connection cannot
/// outlive them.
///
/// # Example
///
/// ```ignore
/// let mut rx = [0u8; 4096];
/// let mut tx = [0u8; 1024];
/// let socket = connector.connect(addr, &mut rx, &mut tx).await?;
/// // socket borrows rx and tx - they cannot be used until socket is dropped
/// ```
///
/// # Note on `async fn` in traits
///
/// We use `async fn` directly here because this trait is designed for embedded
/// single-threaded executors (embassy) where `Send` bounds are not required.
#[allow(async_fn_in_trait)]
pub trait TcpConnector {
    /// The error type returned when a connection fails.
    #[cfg(feature = "defmt")]
    type Error: defmt::Format;
    #[cfg(feature = "log")]
    type Error: core::fmt::Debug;

    /// The established TCP connection type.
    ///
    /// This type must implement `embedded_io_async::Read` and `Write` for
    /// bidirectional communication.
    type Connection<'a>: Read<Error = Self::Error> + Write<Error = Self::Error>
    where
        Self: 'a;

    /// Establish a TCP connection to the given remote address.
    ///
    /// # Arguments
    ///
    /// * `remote` - The socket address (IP + port) to connect to
    /// * `rx_buffer` - Buffer for incoming data (size determines max receive window)
    /// * `tx_buffer` - Buffer for outgoing data (size determines max send window)
    ///
    /// # Returns
    ///
    /// A connected socket that borrows the provided buffers, or an error if
    /// the connection could not be established.
    async fn connect<'a>(
        &'a self,
        remote: SocketAddrV4,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> Result<Self::Connection<'a>, Self::Error>;
}
