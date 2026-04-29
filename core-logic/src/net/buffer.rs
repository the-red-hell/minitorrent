/// Pre-allocated buffers for a single TCP socket.
///
/// These buffers are used by the TCP stack to store incoming and outgoing data.
/// The buffer sizes are configured via const generics at compile time.
///
/// # Const Generic Parameters
///
/// * `RX` - Size of the receive buffer in bytes. Larger buffers allow receiving
///   more data before the application must process it. Typical: 4KB-8KB.
/// * `TX` - Size of the transmit buffer in bytes. Larger buffers allow sending
///   more data before waiting for acknowledgment. Typical: 1KB-2KB.
///
/// # Memory Usage
///
/// Total memory = `RX + TX` bytes. For embedded systems, choose sizes carefully
/// based on available RAM and expected traffic patterns.
///
/// # Example
///
/// ```ignore
/// // 4KB receive, 1KB transmit - suitable for HTTP client
/// let buffers: SocketBuffers<4096, 1024> = SocketBuffers::new();
///
/// // 8KB receive, 2KB transmit - larger buffers for faster transfers
/// let buffers: SocketBuffers<8192, 2048> = SocketBuffers::new();
/// ```
#[defmt_or_log::derive_format_or_debug]
pub(crate) struct SocketBuffers<const RX: usize, const TX: usize> {
    /// Buffer for incoming TCP data (receive window).
    pub(crate) rx: [u8; RX],
    /// Buffer for outgoing TCP data (send window).
    pub(crate) tx: [u8; TX],
}

impl<const RX: usize, const TX: usize> SocketBuffers<RX, TX> {
    /// Create new zeroed socket buffers.
    ///
    /// This is a `const fn` so buffers can be created at compile time
    /// in static contexts if needed.
    pub const fn new() -> Self {
        Self {
            rx: [0u8; RX],
            tx: [0u8; TX],
        }
    }
}

impl<const RX: usize, const TX: usize> Default for SocketBuffers<RX, TX> {
    fn default() -> Self {
        Self::new()
    }
}
