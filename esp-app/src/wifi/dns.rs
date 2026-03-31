use core::net::IpAddr;

use embedded_nal_async::Dns;

use crate::wifi::EspWifi;

impl Dns for EspWifi {
    type Error = embassy_net::dns::Error;

    /// Resolve a hostname to an IP address.
    ///
    /// Only IPv4 is supported in this implementation.
    async fn get_host_by_name(
        &self,
        host: &str,
        addr_type: embedded_nal_async::AddrType,
    ) -> Result<IpAddr, Self::Error> {
        if let embedded_nal_async::AddrType::IPv6 = addr_type {
            return Err(embassy_net::dns::Error::Failed);
        }

        let dns = embassy_net::dns::DnsSocket::new(self.stack);
        let addrs = dns.query(host, embassy_net::dns::DnsQueryType::A).await?;
        let addr = addrs.first().ok_or(embassy_net::dns::Error::Failed)?;

        match addr {
            embassy_net::IpAddress::Ipv4(ipv4_addr) => Ok(IpAddr::V4(*ipv4_addr)),
        }
    }

    async fn get_host_by_address(
        &self,
        _addr: IpAddr,
        _result: &mut [u8],
    ) -> Result<usize, Self::Error> {
        unreachable!("Reverse DNS lookup not used in this application");
    }
}
