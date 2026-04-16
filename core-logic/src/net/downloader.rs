use embedded_nal_async::Dns;

use crate::{
    BitTorrenter, BitTorrenterError, TcpConnector,
    bittorrenter::states::Downloading,
    fs::{FileSystemExt, VolumeMgr},
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
        defmt_or_log::info!("Starting download...");
        let peer = connect_to_peer(
            &mut self.net,
            &mut self.socket_buffers,
            self.state.get_peers()[2],
        )
        .await?;

        let mut handshake_peer = peer
            .into_handshake_performed(self.state.get_info_hash())
            .await
            .map_err(BitTorrenterError::HandshakeFailed)?;

        handshake_peer
            .process_incoming_data()
            .await
            .map_err(BitTorrenterError::TcpError)?;

        // let unchoked_peer = interested_peer
        //     .wait_for_unchoke()
        //     .await
        //     .map_err(BitTorrenterError::TcpError)?;

        // let name = self.state.get_name();

        // defmt_or_log::info!(
        //     "Peer unchoked, starting file download for file {:?}...",
        //     name
        // );
        // self.fs
        //     .open_file(name, embedded_sdmmc::Mode::ReadWriteCreateOrAppend)
        //     .map_err(BitTorrenterError::FsError)?;

        // let piece_length = self.state.get_piece_length();
        // let total_length = self.state.get_total_length();
        // let _finished_peer = unchoked_peer
        //     .download_file(piece_length, total_length, &mut self.fs)
        //     .await
        //     .map_err(BitTorrenterError::TcpError)?;
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
    defmt_or_log::info!("Connecting to peer at: {:?}", peer_addr);

    let conn = net
        .connect(peer_addr, &mut socket_buffers.rx, &mut socket_buffers.tx)
        .await
        .map_err(BitTorrenterError::TcpError)?;

    defmt_or_log::info!("Connected to peer at: {:?}", peer_addr);
    Ok(Peer::new(conn))
}
