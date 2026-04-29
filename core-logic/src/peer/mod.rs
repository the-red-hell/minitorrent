use ::core::marker::PhantomData;

use crate::{TcpConnector, core::PeerId, peer::piece_state::PieceState};
pub(super) mod buf_reader;
pub mod handshake;
pub(crate) mod messages;
mod piece_state;
pub mod process;

pub const BLOCK_SIZE: u32 = 16 * 1024; // 16KB
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
pub(crate) struct Peer<'a, NET, HandshakeState = NotHandshaken>
where
    NET: TcpConnector + 'a,
{
    state: State,
    piece: PieceState,
    connection: NET::Connection<'a>,
    _handshake_state: PhantomData<HandshakeState>,
}

impl<'a, NET, HandshakeState> Peer<'a, NET, HandshakeState>
where
    NET: TcpConnector + 'a,
{
    pub(crate) fn new(connection: NET::Connection<'a>, piece_length: u32, file_size: u32) -> Self {
        Self {
            connection,
            _handshake_state: PhantomData,
            state: State::default(),
            piece: PieceState::new(0, piece_length, file_size),
        }
    }

    // TODO: inline such methods & make them const if possible
    pub(crate) fn connection(&mut self) -> &mut NET::Connection<'a> {
        &mut self.connection
    }
}

#[defmt_or_log::derive_format_or_debug]
#[derive(Clone, Copy, Default)]
pub(super) enum State {
    #[default]
    NotHandshaken,
    ChokedNotInterested,
    ChokedInterested,
    UnchokedInterested,
}

#[defmt_or_log::derive_format_or_debug]
pub(super) struct Handshaken;
#[defmt_or_log::derive_format_or_debug]
pub(super) struct NotHandshaken;
