extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use crate::BLOCK_SIZE;

/// Maximum number of blocks to buffer in memory per piece before writing to disk.
// TODO: if it's more than that, the current implementation of `PieceState` won't work
pub(super) const MAX_BLOCKS: u32 = 5;

/// Represents the state of a piece being downloaded from a peer.
/// The block data is heap-allocated once and reused across pieces.
pub(super) struct PieceState {
    index: u32,
    /// bitfield tracking which blocks have been received
    have: u32,
    /// flat buffer holding block data, allocated once (sized for the largest piece)
    piece: Box<[u8]>,
    /// number of bytes received so far for this piece
    len_bytes: u32,
    /// actual number of blocks for this piece (recomputed on increment)
    num_blocks: u32,
    /// size of the current piece in bytes (last piece of the file may be smaller)
    piece_size: u32,
    /// standard piece size from the torrent metadata
    piece_length: u32,
    /// total file size
    file_size: u32,
}

impl PieceState {
    pub fn new(index: u32, piece_length: u32, file_size: u32) -> Self {
        let piece_size = piece_size_for(index, piece_length, file_size);
        let num_blocks = (piece_size.div_ceil(BLOCK_SIZE)).min(MAX_BLOCKS);
        Self {
            index,
            have: 0,
            // since this is called only once for the first time, `num_blocks` corresponds to the largest number of blocks
            piece: vec![0u8; (num_blocks * BLOCK_SIZE) as usize].into_boxed_slice(),
            len_bytes: 0,
            num_blocks,
            piece_size,
            piece_length,
            file_size: total_length,
        }
    }

    #[inline]
    pub fn get_piece_data(&self) -> &[u8] {
        &self.piece[..self.len_bytes as usize]
    }

    #[inline]
    pub const fn is_complete(&self) -> bool {
        self.have.count_ones() == self.num_blocks
    }

    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    #[inline]
    pub fn add_block(&mut self, begin: u32, block_data: &[u8]) {
        let begin = begin as usize;
        self.piece[begin..begin + block_data.len()].copy_from_slice(block_data);
        self.have |= 1u32 << (begin / BLOCK_SIZE as usize);
        self.len_bytes += block_data.len() as u32;
    }

    pub fn get_next_block_request(&self) -> Option<(u32, u32, u32)> {
        for block_index in 0..self.num_blocks {
            if self.have & (1u32 << block_index) == 0 {
                // we don't have this block, request it
                let begin = block_index * BLOCK_SIZE;
                let block_length = if block_index != self.num_blocks - 1 {
                    BLOCK_SIZE
                } else {
                    // last block might be smaller than BLOCK_SIZE
                    self.piece_size - begin
                };
                return Some((self.index, begin, block_length));
            }
        }
        None
    }

    /// Increments the piece index and resets state for the next piece.
    /// Recomputes `piece_size` and `num_blocks` since the last piece may be smaller.
    #[inline]
    pub(crate) fn increment(&mut self) {
        self.index += 1;
        self.piece_size = piece_size_for(self.index, self.piece_length, self.file_size);
        self.num_blocks = (self.piece_size.div_ceil(BLOCK_SIZE)).min(MAX_BLOCKS);
        self.reset();
    }

    /// Resets the received state without changing the piece index.
    #[inline]
    fn reset(&mut self) {
        self.have = 0;
        self.len_bytes = 0;
    }
}

fn piece_size_for(index: u32, piece_length: u32, total_length: u32) -> u32 {
    let offset = index * piece_length;
    (total_length - offset).min(piece_length)
}
