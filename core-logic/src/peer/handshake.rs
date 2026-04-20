use ::core::marker::PhantomData;

use embedded_io_async::{Read, Write};

use crate::{
    TcpConnector,
    core::InfoHash,
    peer::{Handshaken, NotHandshaken, PEER_ID, Peer},
};

impl<'a, NET> Peer<'a, NET, NotHandshaken>
where
    NET: TcpConnector + 'a,
{
    /// Performs the BitTorrent handshake with the peer.
    /// Returns a new `Peer` instance in the `Handshaken` state if successful.
    #[inline]
    pub(crate) async fn into_handshake_performed(
        mut self,
        info_hash: &InfoHash,
    ) -> Result<Peer<'a, NET, Handshaken>, HandshakeError<NET>> {
        let handshake_msg = construct_handshake(info_hash, &PEER_ID);
        self.connection()
            .write_all(handshake_msg.as_slice())
            .await
            .map_err(HandshakeError::WriteFailed)?;

        let mut response_buf = [0u8; 68];
        self.connection()
            .read_exact(&mut response_buf)
            .await
            .map_err(HandshakeError::ReadFailed)?;

        // only assert Info-Hash, the rest of the handshake response can be different (e.g. reserved bytes, peer_id)
        if response_buf[28..48] != handshake_msg[28..48] {
            return Err(HandshakeError::InvalidHash);
        }

        defmt_or_log::info!("Handshake successful with peer");

        // TODO: send bitfield?

        Ok(Peer {
            connection: self.connection,
            _handshake_state: PhantomData,
            state: crate::peer::State::ChokedNotInterested,
        })
    }
}

#[inline]
fn construct_handshake(info_hash: &InfoHash, peer_id: &[u8; 20]) -> [u8; 68] {
    let mut handshake_msg: [u8; 68] = [0; 68];
    let protocol_str = b"BitTorrent protocol";
    let reserved = [0u8; 8];

    handshake_msg[0] = 19; // Protocol string length
    handshake_msg[1..20].copy_from_slice(protocol_str);
    handshake_msg[20..28].copy_from_slice(&reserved);
    handshake_msg[28..48].copy_from_slice(info_hash);
    handshake_msg[48..68].copy_from_slice(peer_id);

    handshake_msg
}

#[defmt_or_log::derive_format_or_debug]
pub enum HandshakeError<NET>
where
    NET: TcpConnector,
{
    /// Writing has failed
    WriteFailed(NET::Error),
    /// Reading has failed
    ReadFailed(embedded_io_async::ReadExactError<NET::Error>),
    /// Hash mismatch in handshake response
    InvalidHash,
}
