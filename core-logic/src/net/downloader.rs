use embassy_time::Duration;
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
    #[inline]
    pub fn get_peers(&self) -> &[core::net::SocketAddrV4] {
        self.state.get_peers()
    }

    pub async fn download(&mut self) -> Result<(), BitTorrenterError<NET, V>> {
        defmt_or_log::info!("Starting download...");

        let peer = connect_to_valid_peer(
            &mut self.net,
            &mut self.socket_buffers,
            self.state.get_peers(),
            self.state.get_piece_length(),
            self.state.get_total_length(),
        )
        .await?
        .expect("// TODO: no valid peers found");

        defmt_or_log::info!("Connected to peer, performing handshake...");

        let mut handshake_peer = peer
            .into_handshake_performed(self.state.get_info_hash())
            .await
            .map_err(BitTorrenterError::HandshakeFailed)?;

        let name = self.state.get_name();

        self.fs.go_to_root_dir();
        self.fs
            .open_file(name, embedded_sdmmc::Mode::ReadWriteCreateOrAppend)
            .map_err(BitTorrenterError::FsError)?;

        handshake_peer
            .process_incoming_data(&mut self.fs)
            .await
            .map_err(BitTorrenterError::TcpError)?;

        // let unchoked_peer = interested_peer
        //     .wait_for_unchoke()
        //     .await
        //     .map_err(BitTorrenterError::TcpError)?;

        // defmt_or_log::info!(
        //     "Peer unchoked, starting file download for file {:?}...",
        //     name
        // );

        // let piece_length = self.state.get_piece_length();
        // let total_length = self.state.get_total_length();
        // let _finished_peer = unchoked_peer
        //     .download_file(piece_length, total_length, &mut self.fs)
        //     .await
        //     .map_err(BitTorrenterError::TcpError)?;
        Ok(())
    }
}

/// tries to connect to the first peer in a list that responds within a timeout
async fn connect_to_valid_peer<'a, NET, V, const RX: usize, const TX: usize>(
    net: &'a mut NET,
    socket_buffers: &'a mut SocketBuffers<RX, TX>,
    peer_list: &[core::net::SocketAddrV4],
    piece_length: u32,
    file_size: u32,
) -> Result<Option<Peer<'a, NET, NotHandshaken>>, BitTorrenterError<NET, V>>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    for peer_addr in peer_list {
        defmt_or_log::info!("Connecting to peer at: {:?}", peer_addr);

        // SAFETY: On failure paths, the connection (and its borrows) is dropped
        // before the next iteration. On success, we return immediately.
        // The async borrow checker is overly conservative in loops (#63768).
        let (net_ref, rx_ref, tx_ref) = unsafe {
            let net_ref = &mut *(net as *mut NET);
            let rx_ref = &mut *(&mut socket_buffers.rx as *mut [u8; RX] as *mut [u8]);
            let tx_ref = &mut *(&mut socket_buffers.tx as *mut [u8; TX] as *mut [u8]);
            (net_ref, rx_ref, tx_ref)
        };

        let conn = embassy_time::with_timeout(
            Duration::from_secs(20),
            net_ref.connect(*peer_addr, rx_ref, tx_ref),
        )
        .await;

        match conn {
            Ok(Ok(c)) => {
                defmt_or_log::info!("Connected to peer at: {:?}", peer_addr);
                return Ok(Some(Peer::new(c, piece_length, file_size)));
            }
            Ok(Err(_)) => continue,
            Err(_) => {
                defmt_or_log::warn!("Connection to peer timed out");
                continue;
            }
        }
    }
    Ok(None)
}
