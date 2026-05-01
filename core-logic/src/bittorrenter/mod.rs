pub mod error;
pub mod states;
use embedded_nal_async::Dns;

use crate::bittorrenter::states::RequestingTracker;
use crate::net::buffer::SocketBuffers;
use crate::{
    TcpConnector,
    fs::{FileSystem, VolumeMgr},
};

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
///
/// So conceptually:
/// - you give the `BitTorrenter` the SocketBuffer-size, it will create such buffers and own them when it needs to connect (to the tracker + peers)
/// - the `BitTorrenter` calls the implementation of the `TcpConnector` trait, passing the buffers to it
/// - in the current esp-app implementation, this will create `smoltcp`'s `TcpSocket` with the buffers, and smoltcp will use those buffers for the duration of the connection
#[defmt_or_log::derive_format_or_debug]
pub struct BitTorrenter<
    NET,
    V,
    STATE = RequestingTracker,
    const RX: usize = 4096,
    const TX: usize = 1024,
> where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    /// Network implementation for DNS and TCP.
    pub(crate) net: NET,
    /// File system for torrent data.
    pub fs: FileSystem<V>,
    /// Pre-allocated socket buffers owned by this client.
    /// Only one TCP connection can be active at a time.
    pub(crate) socket_buffers: SocketBuffers<RX, TX>,
    /// Unique identifier for this client (sent to trackers and peers).
    pub(crate) peer_id: [u8; 20],
    /// Port number this client listens on for incoming peer connections.
    pub(crate) port: u16,
    pub(crate) state: STATE,
}

impl<NET, V, STATE, const RX: usize, const TX: usize> BitTorrenter<NET, V, STATE, RX, TX>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    /// Get mutable access to the file system.
    pub fn fs(&mut self) -> &mut FileSystem<V> {
        &mut self.fs
    }
}

impl<NET, V, const RX: usize, const TX: usize> BitTorrenter<NET, V, RequestingTracker, RX, TX>
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
            state: RequestingTracker,
        }
    }
}
