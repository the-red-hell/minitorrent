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
    pub(crate) async fn send_interested(
        mut self,
    ) -> Result<Peer<'a, NET, Handshaken, Choked, Interested>, NET::Error> {
        // send interested message to peer

        let interested_msg = PeerMessage::Interested;
        self.connection()
            .write_all(&interested_msg.into_bytes())
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
    pub(crate) async fn wait_for_unchoke(
        mut self,
    ) -> Result<Peer<'a, NET, Handshaken, Unchoked, Interested>, NET::Error> {
        // send interested message to peer

        let mut buf = [0u8; 1];
        while buf[0] != PeerMessageTypes::Unchoke as u8 {
            self.connection().read(&mut buf).await?;
        }

        Ok(Peer {
            connection: self.connection,
            _handshake_state: PhantomData,
            _choke_state: PhantomData,
            _interest_state: PhantomData,
        })
    }
}
