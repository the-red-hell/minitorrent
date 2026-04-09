//! Error Types

use embedded_nal_async::Dns;
use embedded_sdmmc::BlockDevice;

use crate::{TcpConnector, fs::VolumeMgr};

/// Errors that can occur during BitTorrent operations.
///
/// This enum wraps errors from the network stack (DNS/TCP) and file system,
/// allowing callers to handle them uniformly.
#[defmt_or_log::derive_format_or_debug]
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
    FsError(embedded_sdmmc::Error<<<V as VolumeMgr>::BlockDevice as BlockDevice>::Error>),
    /// Failed to parse the tracker's response (e.g., invalid bencoding).
    TrackerResponseParseError,
    /// Failed to perform the BitTorrent handshake with a peer.
    HandshakeFailed,
}
