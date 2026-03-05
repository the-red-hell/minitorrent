use embedded_nal_async::Dns;

use crate::{BitTorrenter, TcpConnector, bittorrenter::states::Downloading, fs::VolumeMgr};

impl<NET, V, const RX: usize, const TX: usize> BitTorrenter<NET, V, Downloading, RX, TX>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    pub fn get_peers(&self) -> &[core::net::SocketAddrV4] {
        self.state.get_peers()
    }
}
