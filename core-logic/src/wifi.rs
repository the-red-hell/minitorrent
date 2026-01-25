use core::net::Ipv4Addr;

#[allow(async_fn_in_trait)]
pub trait WifiStack {
    type Error: defmt::Format;

    async fn make_http_request<'a>(
        &self,
        url: &str,
        rx_buf: &'a mut [u8],
    ) -> Result<&'a mut [u8], Self::Error>;

    fn get_ipv4(&self) -> Ipv4Addr;
}
