#![cfg_attr(not(test), no_std)]

extern crate alloc;

use embedded_nal_async::Dns;
use embedded_sdmmc::BlockDevice;

use crate::fs::{FileSystem, VolumeMgr};

pub mod core;
pub mod fs;
// pub mod wifi;

pub use core::metainfo::{Info, MetaInfoFile};
pub use core::net::TcpConnector;

// ============================================================================
// Socket Buffers
// ============================================================================

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
pub struct SocketBuffers<const RX: usize, const TX: usize> {
    /// Buffer for incoming TCP data (receive window).
    pub rx: [u8; RX],
    /// Buffer for outgoing TCP data (send window).
    pub tx: [u8; TX],
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

// ============================================================================
// BitTorrenter
// ============================================================================

/// The main BitTorrent client that coordinates networking and file system access.
///
/// # Type Parameters
///
/// * `NET` - Network implementation providing DNS resolution and TCP connections.
///   Must implement `TcpConnector` (caller-provided buffers) and `Dns`.
/// * `V` - Volume manager for file system operations (reading/writing torrent data).
/// * `RX` - Socket receive buffer size in bytes (default: 4096).
/// * `TX` - Socket transmit buffer size in bytes (default: 1024).
///
/// # Buffer Ownership
///
/// Unlike designs where the network stack owns socket buffers, `BitTorrenter`
/// owns the buffers and passes them to the network stack when connecting.
/// This avoids interior mutability (RefCell/Mutex) in the network implementation,
/// which is important for embedded systems with limited resources.
///
/// # Example
///
/// ```ignore
/// // Create with default buffer sizes (4KB RX, 1KB TX)
/// let client: BitTorrenter<MyNet, MyVolMgr> = BitTorrenter::new(net, fs);
///
/// // Create with custom buffer sizes
/// let client: BitTorrenter<MyNet, MyVolMgr, 8192, 2048> = BitTorrenter::new(net, fs);
/// ```
pub struct BitTorrenter<NET, V, const RX: usize = 4096, const TX: usize = 1024>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    /// Network implementation for DNS and TCP.
    net: NET,
    /// File system for torrent data.
    fs: FileSystem<V>,
    /// Pre-allocated socket buffers owned by this client.
    /// Only one TCP connection can be active at a time.
    socket_buffers: SocketBuffers<RX, TX>,
    /// Unique identifier for this client (sent to trackers and peers).
    peer_id: [u8; 20],
    /// Port number this client listens on for incoming peer connections.
    port: u16,
}

impl<NET, V, const RX: usize, const TX: usize> BitTorrenter<NET, V, RX, TX>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    /// Create a new BitTorrent client.
    ///
    /// # Arguments
    ///
    /// * `net` - Network implementation (must implement `TcpConnector + Dns`)
    /// * `fs` - File system for reading .torrent files and writing downloaded data
    ///
    /// # Note
    ///
    /// Socket buffers are allocated internally based on the const generic
    /// parameters `RX` and `TX`. Default sizes are 4KB receive, 1KB transmit.
    pub fn new(net: NET, fs: FileSystem<V>) -> Self {
        Self {
            net,
            fs,
            socket_buffers: SocketBuffers::new(),
            peer_id: [0u8; 20],
            port: 6881,
        }
    }

    /// Get mutable access to the file system.
    pub fn fs(&mut self) -> &mut FileSystem<V> {
        &mut self.fs
    }

    /// Get mutable access to the network implementation.
    pub fn net(&mut self) -> &mut NET {
        &mut self.net
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during BitTorrent operations.
///
/// This enum wraps errors from the network stack (DNS/TCP) and file system,
/// allowing callers to handle them uniformly.
#[derive(Debug)]
pub enum BitTorrenterError<NET, V>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    /// DNS resolution failed (e.g., tracker hostname not found).
    DnsError(<NET as Dns>::Error),
    /// TCP connection or I/O failed.
    TcpError(<NET as TcpConnector>::Error),
    /// File system operation failed.
    FsError(<<V as VolumeMgr>::BlockDevice as BlockDevice>::Error),
}
