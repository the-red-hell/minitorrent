use core::net::Ipv4Addr;

use core_logic::wifi::WifiStack;
use embassy_net::{
    Stack,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};

mod network;
pub(crate) mod setup;

pub struct EspWifiStack(Stack<'static>);

impl WifiStack for EspWifiStack {
    type Error = reqwless::Error;
    /// makes a GET request to the provided url and returns the response body
    async fn make_http_request<'a>(
        &self,
        url: &str,
        rx_buf: &'a mut [u8],
    ) -> Result<&'a mut [u8], Self::Error> {
        let state = TcpClientState::<1, 1024, 4096>::new();
        let client = TcpClient::new(self.0, &state);

        let dns = DnsSocket::new(self.0);

        let mut http_client = reqwless::client::HttpClient::new(&client, &dns);

        http_client
            .request(reqwless::request::Method::GET, url)
            .await?
            .send(rx_buf)
            .await?
            .body()
            .read_to_end()
            .await
    }

    fn get_ipv4(&self) -> Ipv4Addr {
        if let Some(config) = self.0.config_v4() {
            config.address.address()
        } else {
            panic!(
                "No IP address found, please check your Wi-Fi connection or whether the setup completed. Restart the device and try again."
            );
        }
    }
}
