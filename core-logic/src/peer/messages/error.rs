#[derive(Debug, defmt::Format)]
pub enum MessageError {
    InvalidMessage,
    InvalidLength,
    UnknownMessageType(u8),
}
