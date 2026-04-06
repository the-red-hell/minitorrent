#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MessageError {
    InvalidMessage,
    InvalidLength,
    UnknownMessageType(u8),
}
