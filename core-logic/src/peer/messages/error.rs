#[defmt_or_log::derive_format_or_debug]
pub enum MessageError {
    Empty,
    _InvalidMessage,
    InvalidLength,
    UnknownMessageType(u8),
}
