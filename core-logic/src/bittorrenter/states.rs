use heapless::Vec;

pub struct RequestingTracker;

pub struct Downloading {
    _peers: Vec<core::net::SocketAddrV4, 10>,
}
impl Downloading {
    pub(crate) fn new(peers: Vec<core::net::SocketAddrV4, 10>) -> Self {
        Self { _peers: peers }
    }

    pub(crate) fn get_peers(&self) -> &[core::net::SocketAddrV4] {
        &self._peers
    }
}

pub struct Seeding;
