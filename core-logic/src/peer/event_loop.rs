use ::core::marker::PhantomData;

use embedded_io_async::{Read, Write};

use crate::{
    TcpConnector,
    peer::{
        Choked, Handshaken, Interested, NotInterested, Peer, Unchoked,
        messages::{PeerMessageTypes, incoming::PeerMessage},
    },
};

impl<'a, NET> Peer<'a, NET, Handshaken, Choked, NotInterested>
where
    NET: TcpConnector + 'a,
{
    /// Sends an interested message to the peer, indicating that we want to download pieces.
    #[inline]
    pub(crate) async fn send_interested(
        mut self,
    ) -> Result<Peer<'a, NET, Handshaken, Choked, Interested>, NET::Error> {
        // send interested message to peer

        let interested_msg = PeerMessage::Interested;
        self.connection()
            .write_all(&interested_msg.as_bittorrent_bytes())
            .await?;

        Ok(Peer {
            connection: self.connection,
            _handshake_state: PhantomData,
            _choke_state: PhantomData,
            _interest_state: PhantomData,
        })
    }
}

impl<'a, NET> Peer<'a, NET, Handshaken, Choked, Interested>
where
    NET: TcpConnector + 'a,
{
    /// Waits for an unchoke message from the peer. This indicates that the peer is now willing to send data.
    #[inline]
    pub(crate) async fn wait_for_unchoke(
        mut self,
    ) -> Result<Peer<'a, NET, Handshaken, Unchoked, Interested>, NET::Error> {
        // send interested message to peer

        let mut buf = [0u8; 5]; // the unchoke message is 5 bytes long, for optimization we read it directly into those 5 bytes
        while !matches!(
            PeerMessage::from_bytes(&buf),
            Err(_) | Ok(Some(PeerMessage::Unchoke))
        ) {
            self.connection()
                .read_exact(&mut buf) // TODO: I cannot read exactly 5 bytes, I have to call from_bytes until it finally returns something useful (Ok(Some(...)))
                .await
                .map_err(|read_exact_error| match read_exact_error {
                    ReadExactError::UnexpectedEof => todo!("fuking implement this"),
                    ReadExactError::Other(e) => e,
                })?;
        }

        Ok(Peer {
            connection: self.connection,
            _handshake_state: PhantomData,
            _choke_state: PhantomData,
            _interest_state: PhantomData,
        })
    }
}

        }

        Ok(Peer {
            connection: self.connection,
            _handshake_state: PhantomData,
            _choke_state: PhantomData,
            _interest_state: PhantomData,
        })
    }
}
