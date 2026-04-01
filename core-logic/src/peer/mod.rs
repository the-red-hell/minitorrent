use ::core::marker::PhantomData;

use crate::{TcpConnector, core::PeerId};
pub mod event_loop;
mod handshake;
mod messages;

pub const BLOCK_SIZE: usize = 16 * 1024; // 16KB
const PEER_ID: PeerId = *b"AwesomeESP32C3Client";

/// A Peer in the BitTorrent protocol, parameterized by its handshake, choke, and interest states.
///
/// Flow:
///     NotHandshaken -> Handshaken (via peer.into_handshake_performed())
///     Choked <-> Unchoked
///     Interested <-> NotInterested
///     
/// ```ignore
/// // Create a new peer connection, the tcp-connection comes from the `BitTorrenter`
/// let peer = Peer::new(tcp_connection); // NotHandshaken, Choked, NotInterested
/// // Perform handshake
/// let peer = peer.into_handshake_performed(info_hash).await?; // Handshaken, Choked, NotInterested
/// // ... later ...
/// let peer = peer.unchoke();```
pub(crate) struct Peer<
    'a,
    NET,
    HandsShaken = NotHandshaken,
    CHOKED = Choked,
    INTERESTED = NotInterested,
> where
    NET: TcpConnector + 'a,
{
    connection: NET::Connection<'a>,
    _handshake_state: PhantomData<HandsShaken>,
    _choke_state: PhantomData<CHOKED>,
    _interest_state: PhantomData<INTERESTED>,
}

impl<'a, NET, HandsShaken, CHOKED, INTERESTED> Peer<'a, NET, HandsShaken, CHOKED, INTERESTED>
where
    NET: TcpConnector + 'a,
{
    pub(crate) fn new(connection: NET::Connection<'a>) -> Self {
        Self {
            connection,
            _handshake_state: PhantomData,
            _choke_state: PhantomData,
            _interest_state: PhantomData,
        }
    }

    pub(crate) fn connection(&mut self) -> &mut NET::Connection<'a> {
        &mut self.connection
    }
}

#[derive(Debug)]
pub(super) struct Choked;
#[derive(Debug)]
pub(super) struct Unchoked;

#[derive(Debug)]
pub(super) struct Interested;
#[derive(Debug)]
pub(super) struct NotInterested;

#[derive(Debug)]
pub(super) struct Handshaken;
#[derive(Debug)]
pub(super) struct NotHandshaken;
