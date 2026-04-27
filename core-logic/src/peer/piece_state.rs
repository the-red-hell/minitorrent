use crate::BLOCK_SIZE;

/// A struct representing the state of a piece being downloaded from a peer. (1 -> 1)
/// We need some weird workarounds to make it const.
/// the parameter MAX_N represents the number of blocks to receive at which we will definitely write to disk
pub(super) struct PieceState<const MAX_N: usize> {
    index: u32,
    have: [bool; MAX_N],
    piece: [[u8; BLOCK_SIZE]; MAX_N],
    /// used to write the piece to the file system once it's complete, (last one might not be full, so we can't just flatten the piece array)
    len_bytes: usize,
    /// used to check if we received all the blocks for a piece, or at least MAX_N blocks, since we don't know how many blocks the piece contains at all
    num_blocks: usize,
}

impl<const MAX_N: usize> PieceState<MAX_N> {
    #[inline]
    // TODO: we don't even know how much blocks the piece contains at all
    /// Creates a new `PieceState` for the given piece index and file size.
    /// If there are already blocks received for the piece, they can be passed in the `have` array.
    pub fn new(index: u32, file_size: u32, have: [bool; MAX_N]) -> Self {
        let num_blocks = file_size.div_ceil(BLOCK_SIZE as u32) as usize;
        let num_blocks = if num_blocks > MAX_N {
            MAX_N
        } else {
            num_blocks
        };

        // if we already have some blocks, calculate the length of the piece that we have
        let have_count = have
            .iter()
            .take(num_blocks)
            .filter(|&&have_block| have_block)
            .count();
        let len_bytes = have_count * BLOCK_SIZE;

        Self {
            index,
            have,
            piece: [[0; BLOCK_SIZE]; MAX_N],
            len_bytes,
            num_blocks,
        }
    }

    #[inline]
    pub const fn num_blocks(&self) -> usize {
        self.num_blocks
    }

    #[inline]
    pub fn have_count(&self) -> usize {
        self.have
            .iter()
            .take(self.num_blocks)
            .filter(|&&have_block| have_block)
            .count()
    }

    #[inline]
    pub fn get_piece_data(&self) -> &[u8] {
        &self.piece.as_flattened()[..self.len_bytes]
    }

    /// Checks whether we received all the blocks for a piece or MAX_N blocks
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.have
            .iter()
            .take(self.num_blocks)
            .all(|&have_block| have_block)
    }

    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    #[inline]
    pub fn add_block(&mut self, begin: usize, block_data: &[u8]) {
        let block_index = begin / BLOCK_SIZE;
        self.piece[block_index][..block_data.len()].copy_from_slice(block_data);
        self.mark_block_as_have(block_index);
        self.len_bytes += block_data.len();
    }

    pub fn get_next_block_request(&self) -> Option<(u32, u32, u32)> {
        for block_index in 0..self.num_blocks {
            if !self.have[block_index] {
                let begin = block_index * BLOCK_SIZE;
                let block_length = if block_index != self.num_blocks - 1 {
                    BLOCK_SIZE
                } else {
                    // last block might be smaller than BLOCK_SIZE
                    (self.len_bytes - begin).min(BLOCK_SIZE)
                };
                return Some((self.index, begin as u32, block_length as u32));
            }
        }
        None
    }

    #[inline]
    const fn mark_block_as_have(&mut self, block_index: usize) {
        self.have[block_index] = true;
    }

    /// increments the piece-index and resets the have array and the length
    #[inline]
    pub(crate) const fn increment(&mut self) {
        self.index += 1;
        self.reset();
    }

    #[inline]
    /// resets the have array and the length
    pub(crate) const fn reset(&mut self) {
        self.have = [false; MAX_N];
        self.len_bytes = 0;
    }
}
