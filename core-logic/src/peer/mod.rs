use ::core::marker::PhantomData;

use crate::{TcpConnector, core::PeerId, peer::piece_state::PieceState};
pub(super) mod buf_reader;
pub mod handshake;
mod messages;
mod piece_state;
pub mod process;

pub const BLOCK_SIZE: usize = 16 * 1024; // 16KB
const PEER_ID: PeerId = *b"AwesomeESP32C3Client";
const NUM_BLOCKS_PER_PIECE: usize = 5; // TODO: maybe justify this number

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
    piece: PieceState<NUM_BLOCKS_PER_PIECE>,
    file_size: u32,
    connection: NET::Connection<'a>,
    _handshake_state: PhantomData<HandshakeState>,
}

impl<'a, NET, HandshakeState> Peer<'a, NET, HandshakeState>
where
    NET: TcpConnector + 'a,
{
    pub(crate) fn new(connection: NET::Connection<'a>, file_size: u32) -> Self {
        Self {
            connection,
            _handshake_state: PhantomData,
            state: State::default(),
            file_size,
            piece: PieceState::new(0, file_size, [false; NUM_BLOCKS_PER_PIECE]),
        }
    }

    // TODO: inline such methods & make them const if possible
    pub(crate) fn connection(&mut self) -> &mut NET::Connection<'a> {
        &mut self.connection
    }

    #[inline]
    pub(crate) const fn file_size(&self) -> u32 {
        self.file_size
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
