#[defmt_or_log::derive_format_or_debug]
pub enum MessageError {
    _InvalidMessage,
    InvalidLength,
    UnknownMessageType(u8),
}
