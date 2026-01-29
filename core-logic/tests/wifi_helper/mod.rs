use std::net::Ipv4Addr;

use core_logic::wifi::WifiStack;

pub const IP_ADDRESS: &Ipv4Addr = &std::net::Ipv4Addr::new(192, 168, 1, 42);

#[derive(Debug)]
pub struct WifiError(pub std::io::Error);

impl defmt::Format for WifiError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "WifiError: {:?}", defmt::Debug2Format(&self.0));
    }
}

impl From<std::io::Error> for WifiError {
    fn from(err: std::io::Error) -> Self {
        WifiError(err)
    }
}

pub struct WifiStackDuple;

impl WifiStack for WifiStackDuple {
    type Error = WifiError;

    async fn make_http_request<'a>(
        &self,
        url: &str,
        rx_buf: &'a mut [u8],
    ) -> Result<&'a mut [u8], Self::Error> {
        let response = reqwest::get(url).await.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP request error: {}", e),
            )
        })?;
        let bytes = response.bytes().await.map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("HTTP read error: {}", e))
        })?;

        let len = bytes.len().min(rx_buf.len());
        rx_buf[..len].copy_from_slice(&bytes[..len]);
        Ok(&mut rx_buf[..len])
    }

    fn get_ipv4(&self) -> std::net::Ipv4Addr {
        *IP_ADDRESS
    }
}
