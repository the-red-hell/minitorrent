use embedded_io_async::{Read, Write};

use crate::{
    TcpConnector,
    fs::{FileSystem, FileSystemExt, VolumeMgr},
    peer::{BLOCK_SIZE, Handshaken, Peer, State, buf_reader::BufReader, messages::PeerMessage},
};

impl<'a, NET> Peer<'a, NET, Handshaken>
where
    NET: TcpConnector + 'a,
{
    /// main entry
    /// - reads data
    /// - parses & handles messages
    pub(crate) async fn process_incoming_data(
        &mut self,
        fs: &mut FileSystem<impl VolumeMgr>,
    ) -> Result<(), NET::Error> {
        let mut buf = BufReader::<
            {
                BLOCK_SIZE as usize + 4 /* length */ + 1 /* id */
            },
        >::new();

        loop {
            // read data from the peer connection into the buffer
            match self.connection().read(buf.remaining_mut()).await {
                Ok(0) => {
                    defmt_or_log::info!("Peer closed the connection");
                    break; // TODO
                }
                Ok(_) => (),
                Err(e) => {
                    defmt_or_log::error!("Failed to read from peer: {:?}", e);
                    break; // TODO
                }
            }

            // advance the buffer's length by the number of bytes read
            let bytes_read = buf.len();
            buf.advance_n(bytes_read);

            // try to parse a message from the buffer
            let msg = match PeerMessage::from_bytes(&mut buf) {
                Ok(Some(msg)) => {
                    defmt_or_log::info!("Received message from peer: {:?}", msg);
                    msg
                }
                Ok(None) => continue,
                Err(e) => {
                    defmt_or_log::warn!(
                        "Failed to parse peer message: {:?}. Ignoring and waiting for more data.",
                        e
                    );
                    continue;
                }
            };

            // process the message
            self.process_msg(&msg, fs).await?;

            // reset the buffer for the next message
            buf.reset();
        }

        Ok(())
    }
    /// Processes an incoming peer message.
    async fn process_msg(
        &mut self,
        msg: &PeerMessage<'_>,
        fs: &mut FileSystem<impl VolumeMgr>,
    ) -> Result<(), NET::Error> {
        match (self.state, msg) {
            (State::NotHandshaken, _) => {
                unreachable!("this method isn't callable here");
            }
            (State::ChokedNotInterested, _) => {
                let msg = PeerMessage::Interested;
                self.connection()
                    .write_all(&msg.as_bittorrent_bytes())
                    .await?;
            }
            (State::ChokedInterested, PeerMessage::Unchoke) => {
                self.state = State::UnchokedInterested;
                self.send_request().await?;
            }
            (
                State::UnchokedInterested,
                PeerMessage::Piece {
                    index,
                    begin,
                    block,
                },
            ) => {
                self.handle_piece_message(*index, *begin, block, fs).await?;
                self.send_request().await?;
            }
            (State::UnchokedInterested, PeerMessage::Choke) => {
                self.state = State::ChokedInterested;
            }
            _ => todo!(),
        }

        Ok(())
    }

    async fn send_request(&mut self) -> Result<(), NET::Error> {
        let (index, begin, length) = match self.piece.get_next_block_request() {
            Some(req) => req,
            None => {
                defmt_or_log::info!(
                    "No more blocks to request for piece {}, waiting for piece to be complete before requesting the next one.",
                    self.piece.index()
                );
                return Ok(());
            }
        };
        let req_msg = PeerMessage::Request {
            index,
            begin,
            length,
        };

        self.connection()
            .write_all(&req_msg.as_bittorrent_bytes())
            .await?;

        defmt_or_log::trace!(
            "Requested block at begin: {} from piece {} from peer",
            begin,
            index
        );
        Ok(())
    }

    async fn handle_piece_message(
        &mut self,
        index: u32,
        begin: u32,
        block: &[u8],
        fs: &mut FileSystem<impl VolumeMgr>,
    ) -> Result<(), NET::Error> {
        defmt_or_log::trace!(
            "Received block at begin: {} from piece {} from peer",
            begin,
            index
        );

        // wrong index
        if index != self.piece.index() {
            defmt_or_log::warn!(
                "Received block for piece {}, but currently processing piece {}. Ignoring block.",
                index,
                self.piece.index()
            );
            return Ok(());
        }

        // add block
        self.piece.add_block(begin, block);
        // TODO: update SHA1

        // check whether complete
        if self.piece.is_complete() {
            defmt_or_log::info!(
                "Received complete piece {}, writing to file system...",
                self.piece.index()
            );

            // TODO: check SHA1

            // TODO: seek?

            fs.write_to_opened_file(self.piece.get_piece_data())
                .await
                .expect("Failed to write piece to file system");

            // move onto the next piece
            self.piece.increment();
        }

        Ok(())
    }
}

// impl<'a, NET> Peer<'a, NET, Handshaken, Choked, NotInterested>
// where
//     NET: TcpConnector + 'a,
// {
//     /// Sends an interested message to the peer, indicating that we want to download pieces.
//     #[inline]
//     pub(crate) async fn send_interested(
//         mut self,
//     ) -> Result<Peer<'a, NET, Handshaken, Choked, Interested>, NET::Error> {
//         // send interested message to peer

//         let interested_msg = PeerMessage::Interested;
//         self.connection()
//             .write_all(&interested_msg.as_bittorrent_bytes())
//             .await?;

//         Ok(Peer {
//             connection: self.connection,
//             _handshake_state: PhantomData,
//             _choke_state: PhantomData,
//             _interest_state: PhantomData,
//         })
//     }
// }

// impl<'a, NET> Peer<'a, NET, Handshaken, Choked, Interested>
// where
//     NET: TcpConnector + 'a,
// {
//     /// Waits for an unchoke message from the peer. This indicates that the peer is now willing to send data.
//     #[inline]
//     pub(crate) async fn wait_for_unchoke(
//         mut self,
//     ) -> Result<Peer<'a, NET, Handshaken, Unchoked, Interested>, NET::Error> {
//         // send interested message to peer

//         let mut buf = [0u8; 5]; // the unchoke message is 5 bytes long, for optimization we read it directly into those 5 bytes
//         while !matches!(
//             PeerMessage::from_bytes(&buf),
//             Err(_) | Ok(Some(PeerMessage::Unchoke))
//         ) {
//             self.connection()
//                 .read_exact(&mut buf) // TODO: I cannot read exactly 5 bytes, I have to call from_bytes until it finally returns something useful (Ok(Some(...)))
//                 .await
//                 .map_err(|read_exact_error| match read_exact_error {
//                     ReadExactError::UnexpectedEof => todo!("fuking implement this"),
//                     ReadExactError::Other(e) => e,
//                 })?;
//         }

//         Ok(Peer {
//             connection: self.connection,
//             _handshake_state: PhantomData,
//             _choke_state: PhantomData,
//             _interest_state: PhantomData,
//         })
//     }
// }

// impl<'a, NET> Peer<'a, NET, Handshaken, Unchoked, Interested>
// where
//     NET: TcpConnector + 'a,
// {
//     pub(crate) async fn download_file<V>(
//         mut self,
//         piece_length: u32,
//         total_length: u32,
//         fs: &mut FileSystem<V>,
//     ) -> Result<Peer<'a, NET, Handshaken, Unchoked, NotInterested>, NET::Error>
//     where
//         V: VolumeMgr,
//     {
//         let mut piece_i: u32 = 0;
//         let num_pieces = total_length / piece_length
//             + if total_length.is_multiple_of(piece_length) {
//                 0
//             } else {
//                 1
//             };
//         let mut buf = [0u8; BLOCK_SIZE];

//         while piece_i <= num_pieces {
//             let length = if piece_i == num_pieces {
//                 total_length - piece_i * piece_length
//             } else {
//                 piece_length
//             };
//             let req_msg = PeerMessage::Request {
//                 index: piece_i,
//                 begin: piece_i * BLOCK_SIZE as u32,
//                 length,
//             };

//             self.connection()
//                 .write_all(&req_msg.as_bittorrent_bytes())
//                 .await?;
//             defmt_or_log::info!("Requested piece {} from peer", piece_i);

//             self.connection()
//                 .read_exact(&mut buf[..length as usize])
//                 .await
//                 .map_err(|read_exact_error| match read_exact_error {
//                     ReadExactError::UnexpectedEof => {
//                         panic!("reached unexpected end of file")
//                     }
//                     ReadExactError::Other(e) => e,
//                 })?;

//             let res = match PeerMessage::from_bytes(&buf[..length as usize]) {
//                 Ok(msg) => msg,
//                 Err(e) => {
//                     defmt_or_log::warn!(
//                         "Failed to parse peer message: {:?}. Ignoring and moving on to the next piece.",
//                         e
//                     );
//                     continue;
//                 }
//             };

//             match res {
//                 Some(PeerMessage::Piece { index, block, .. }) => {
//                     // TODOO
//                     // we got the piece, move on to the next one
//                     defmt_or_log::info!("Received piece {} from peer", index);
//                     // TODO: update SHA1 hash of the piece and verify it matches the expected hash from the torrent metadata

//                     fs.write_to_opened_file(block).await.expect("let's see");
//                 }
//                 msg => {
//                     defmt_or_log::warn!(
//                         "Expected piece message, got something else: {:?}. Ignoring and moving on to the next piece.",
//                         msg
//                     );
//                 }
//             }

//             piece_i += 1;
//         }

//         Ok(Peer {
//             connection: self.connection,
//             _handshake_state: PhantomData,
//             _choke_state: PhantomData,
//             _interest_state: PhantomData,
//         })
//     }
// }
