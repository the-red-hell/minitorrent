use ::core::marker::PhantomData;

use crate::{TcpConnector, core::PeerId};
pub(super) mod buf_reader;
pub mod handshake;
mod messages;
pub mod process;

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
pub(crate) struct Peer<'a, NET, HandshakeState = NotHandshaken>
where
    NET: TcpConnector + 'a,
{
    state: State,
    piece: PieceState<5>, // TODO: don't hardcode number of blocks
    connection: NET::Connection<'a>,
    _handshake_state: PhantomData<HandshakeState>,
}

impl<'a, NET, HandshakeState> Peer<'a, NET, HandshakeState>
where
    NET: TcpConnector + 'a,
{
    pub(crate) fn new(connection: NET::Connection<'a>) -> Self {
        Self {
            connection,
            _handshake_state: PhantomData,
            state: State::default(),
            piece: PieceState::default(),
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

struct PieceState<const N: usize> {
    index: usize,
    have: [bool; N],
    piece: [[u8; BLOCK_SIZE]; N],
    /// used to write the piece to the file system once it's complete, (last one might not be full)
    len: usize,
}

impl<const N: usize> Default for PieceState<N> {
    #[inline]
    fn default() -> Self {
        Self {
            index: 0,
            have: [false; N],
            piece: [[0; BLOCK_SIZE]; N],
            len: 0,
        }
    }
}

impl<const N: usize> PieceState<N> {
    #[inline]
    // TODO: we don't even know how much blocks the piece contains at all
    const fn new(index: usize) -> Self {
        Self {
            index,
            have: [false; N],
            piece: [[0; BLOCK_SIZE]; N],
            len: 0,
        }
    }

    #[inline]
    const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        self.have.iter().all(|&have_block| have_block)
    }

    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn add_block(&mut self, begin: usize, block_data: &[u8]) {
        let block_index = begin / BLOCK_SIZE;
        self.piece[block_index][..block_data.len()].copy_from_slice(block_data);
        self.mark_block_as_have(block_index);
        self.len += block_data.len();
    }

    #[inline]
    const fn mark_block_as_have(&mut self, block_index: usize) {
        self.have[block_index] = true;
    }
}
