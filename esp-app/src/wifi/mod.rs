use core::net::SocketAddrV4;
use core_logic::TcpConnector;
use embassy_net::{Stack, tcp::TcpSocket};

use crate::wifi::{error::TcpError, socket::EspTcpSocket};

pub mod dns;
pub mod error;
mod network;
pub(crate) mod setup;
pub mod socket;

/// WiFi network client for ESP32 that provides DNS resolution and TCP connections.
///
/// This struct only holds a reference to the
/// embassy-net `Stack`. **It does not own TCP socket buffers.** Buffers
/// are provided by the caller (e.g., `BitTorrenter`) when establishing connections.
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

impl TcpConnector for EspWifi {
    type Error = TcpError;
    type Connection<'a> = EspTcpSocket<'a>;

    async fn connect<'a>(
        &'a self,
        remote: SocketAddrV4,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> Result<Self::Connection<'a>, Self::Error> {
        let mut socket = TcpSocket::new(self.stack, rx_buffer, tx_buffer);
        socket.connect(remote).await?;
        Ok(EspTcpSocket::new(socket))
    }
}
