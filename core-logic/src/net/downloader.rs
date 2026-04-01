use embedded_nal_async::Dns;

use crate::{
    BitTorrenter, BitTorrenterError, TcpConnector,
    bittorrenter::states::Downloading,
    fs::VolumeMgr,
    net::buffer::SocketBuffers,
    peer::{NotHandshaken, Peer},
};

impl<NET, V, const RX: usize, const TX: usize> BitTorrenter<NET, V, Downloading, RX, TX>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    pub fn get_peers(&self) -> &[core::net::SocketAddrV4] {
        self.state.get_peers()
    }

    pub async fn download(&mut self) -> Result<(), BitTorrenterError<NET, V>> {
        defmt::info!("Starting download...");
        let peer = connect_to_peer(
            &mut self.net,
            &mut self.socket_buffers,
            self.state.get_peers()[0],
        )
        .await?;
        let handshake_peer = peer
            .into_handshake_performed(self.state.get_info_hash())
            .await
            .map_err(|_| BitTorrenterError::HandshakeFailed)?;

        let interested_peer = handshake_peer
            .send_interested()
            .await
            .map_err(BitTorrenterError::TcpError)?;

        let unchoked_peer = interested_peer
            .wait_for_unchoke()
            .await
            .map_err(BitTorrenterError::TcpError)?;
        Ok(())
    }
}

async fn connect_to_peer<'a, NET, V, const RX: usize, const TX: usize>(
    net: &'a mut NET,
    socket_buffers: &'a mut SocketBuffers<RX, TX>,
    peer_addr: core::net::SocketAddrV4,
) -> Result<Peer<'a, NET, NotHandshaken>, BitTorrenterError<NET, V>>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    let conn = net
        .connect(peer_addr, &mut socket_buffers.rx, &mut socket_buffers.tx)
        .await
        .map_err(BitTorrenterError::TcpError)?;

    defmt::info!("Connected to peer at: {:?}", peer_addr);
    Ok(Peer::new(conn))
}
