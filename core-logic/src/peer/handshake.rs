use ::core::marker::PhantomData;

use embedded_io_async::Write;
use heapless::Vec;

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
    pub(crate) async fn into_handshake_performed(
        mut self,
        info_hash: &InfoHash,
    ) -> Result<Peer<'a, NET, Handshaken>, HandshakeError> {
        let mut handshake_msg: Vec<u8, 68> = Vec::new();
        let protocol_str = b"BitTorrent protocol";
        let reserved = [0u8; 8];

        handshake_msg.extend_from_slice(b"19").unwrap();
        handshake_msg.extend_from_slice(protocol_str).unwrap();
        handshake_msg.extend_from_slice(&reserved).unwrap();
        handshake_msg.extend_from_slice(info_hash).unwrap();
        handshake_msg.extend_from_slice(&PEER_ID).unwrap();

        self.connection()
            .write_all(handshake_msg.as_slice())
            .await
            .map_err(|_| HandshakeError)?;

        Ok(Peer {
            connection: self.connection,
            _handshake_state: PhantomData,
            _choke_state: PhantomData,
            _interest_state: PhantomData,
        })
    }
}

pub(crate) struct HandshakeError;
