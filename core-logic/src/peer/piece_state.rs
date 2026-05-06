use static_cell::StaticCell;

use crate::BLOCK_SIZE;

/// Maximum number of blocks to buffer in memory per piece before writing to disk.
pub(super) const NUM_BLOCKS: u32 = 2;

/// Represents the state of a piece being downloaded from a peer.
/// The block data lives in a static buffer to avoid heap fragmentation.
pub(super) struct PieceState {
    /// current piece index (0-based)
    index: u32,
    /// bitfield tracking which blocks have been received
    have: u32,
    /// reference to a static buffer holding one block's worth of data
    piece: &'static mut [u8; (NUM_BLOCKS * BLOCK_SIZE) as usize],
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
    pub(super) fn new(index: u32, piece_length: u32, file_size: u32) -> Self {
        static PIECE_BUF: StaticCell<[u8; (NUM_BLOCKS * BLOCK_SIZE) as usize]> = StaticCell::new();
        let piece_size = piece_size_for(index, piece_length, file_size);
        let num_blocks = piece_size.div_ceil(BLOCK_SIZE);
        Self {
            index,
            have: 0,
            piece: PIECE_BUF.init([0u8; (NUM_BLOCKS * BLOCK_SIZE) as usize]),
            len_bytes: 0,
            num_blocks,
            piece_size,
            piece_length,
            file_size,
        }
    }

    pub(super) fn get_piece_data(&self) -> &[u8] {
        &self.piece.as_slice()[..self.len_bytes as usize]
    }

    /// returns if all blocks for this piece have been received or the buffer for this piece is full
    pub(super) const fn should_write(&self) -> bool {
        self.have.count_ones() == self.num_blocks || self.len_bytes == NUM_BLOCKS * BLOCK_SIZE
    }

    pub(super) const fn index(&self) -> u32 {
        self.index
    }

    pub(super) fn add_block(&mut self, begin: u32, block_data: &[u8]) {
        self.piece[self.len_bytes as usize..self.len_bytes as usize + block_data.len()]
            .copy_from_slice(block_data);
        // here we need the real begin offset to set the bitfield
        self.have |= 1u32 << (begin as usize / BLOCK_SIZE as usize);
        self.len_bytes += block_data.len() as u32;
    }

    /// returns the index, begin and length of the next block to request, or None if all blocks have been received
    pub(super) fn get_next_block_request(&self) -> Option<(u32, u32, u32)> {
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
    ///
    /// Returns whether there are more pieces to request.
    pub(super) fn increment(&mut self) -> bool {
        if !self.is_complete() {
            // the piece isn't finished yet, but the buffer is full
            self.len_bytes = 0;
            return true;
        }
        if (self.index() + 1) * self.piece_length >= self.file_size {
            // no more pieces to request
            return false;
        }
        self.index += 1;
        self.piece_size = piece_size_for(self.index, self.piece_length, self.file_size);
        self.num_blocks = self.piece_size.div_ceil(BLOCK_SIZE);

        self.reset();
        true
    }

    pub(super) const fn num_pieces(&self) -> u32 {
        self.file_size.div_ceil(self.piece_length)
    }

    //
    const fn is_complete(&self) -> bool {
        self.have.count_ones() == self.num_blocks
    }

    /// Resets the received state without changing the piece index.
    const fn reset(&mut self) {
        self.have = 0;
        self.len_bytes = 0;
    }
}

const fn piece_size_for(index: u32, piece_length: u32, file_size: u32) -> u32 {
    let offset = index * piece_length;
    // workaround, since `min()` isn't const yet
    // basically `(file_size - offset).min(piece_length)`
    if file_size - offset < piece_length {
        file_size - offset
    } else {
        piece_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_state() {
        let piece_size: u32 = NUM_BLOCKS * BLOCK_SIZE; // 32KB
        let file_size: u32 = piece_size * 2 + 1024; // 2 pieces: 64KB, 1KB

        let mut piece_state = PieceState::new(0, piece_size, file_size);
        assert_eq!(piece_state.piece_size, piece_size);
        assert_eq!(piece_state.num_blocks, NUM_BLOCKS);
        assert!(!piece_state.should_write());
        assert_eq!(
            piece_state.get_next_block_request(),
            Some((0, 0, BLOCK_SIZE))
        );

        // Simulate receiving blocks
        // PIECE 0
        assert_eq!(piece_state.index(), 0);
        // block 0
        piece_state.add_block(0, &[0u8; (NUM_BLOCKS * BLOCK_SIZE) as usize]);
        // buffer is full
        assert!(piece_state.should_write());
        assert!(piece_state.increment());
        assert_eq!(
            piece_state.get_next_block_request(),
            Some((0, BLOCK_SIZE, BLOCK_SIZE))
        );
        // block 1
        piece_state.add_block(BLOCK_SIZE, &[0u8; BLOCK_SIZE as usize]);
        assert!(piece_state.should_write());
        piece_state.increment();

        // PIECE 1
        assert_eq!(piece_state.index(), 1);
        assert_eq!(
            piece_state.get_next_block_request(),
            Some((1, 0, BLOCK_SIZE))
        );
        // block 0
        piece_state.add_block(0, &[0u8; (NUM_BLOCKS * BLOCK_SIZE) as usize]);
        // buffer is full
        assert!(piece_state.should_write());
        assert!(piece_state.increment());
        assert_eq!(
            piece_state.get_next_block_request(),
            Some((1, BLOCK_SIZE, BLOCK_SIZE))
        );
        // block 1
        piece_state.add_block(BLOCK_SIZE, &[0u8; BLOCK_SIZE as usize]);
        assert!(piece_state.should_write());
        piece_state.increment();

        // PIECE 2
        // go to next piece
        assert_eq!(piece_state.index(), 2);
        assert_eq!(piece_state.piece_size, 1024);
        assert_eq!(piece_state.num_blocks, 1);
        assert!(!piece_state.should_write());
        assert_eq!(piece_state.get_next_block_request(), Some((2, 0, 1024)));

        // receive first and only block for piece 2
        piece_state.add_block(0, &[0u8; 1024]);
        assert!(piece_state.should_write());
        piece_state.increment();
        assert!(piece_state.should_write());

        // no more pieces to request
        piece_state.increment();
        assert_eq!(piece_state.get_next_block_request(), None);
    }
}
