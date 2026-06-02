use embedded_nal_async::Dns;

use crate::{
    BitTorrenter, BitTorrenterError, TcpConnector, bittorrenter::states::Seeding, fs::VolumeMgr,
};

impl<NET, V, const RX: usize, const TX: usize> BitTorrenter<NET, V, Seeding, RX, TX>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    pub async fn _seed(&mut self) -> Result<(), BitTorrenterError<NET, V>> {
        Ok(())
    }
}
