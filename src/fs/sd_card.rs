use embedded_hal::spi::SpiBus;
use embedded_sdmmc::SdCard;
use esp_hal::{
    Async, Blocking,
    peripherals::DMA_CH0,
    spi::master::{Spi, SpiDma},
};

mod esp_spi_dma {
    use esp_hal::{
        Async,
        dma::{DmaRxBuf, DmaRxStreamBuf, DmaTransferRx, DmaTransferRxTx, DmaTxBuf},
        dma_rx_stream_buffer,
        spi::master::SpiDma,
    };

    pub struct EspSpiDma<'d> {
        // Wrapped in Option so we can move them into the async driver during transfer
        spi: Option<SpiDma<'d, Async>>,
        tx_buf: Option<DmaTxBuf>,
        rx_buf: Option<DmaRxBuf>,
    }

    impl<'d> EspSpiDma<'d> {
        pub fn new(spi: SpiDma<'d, Async>, tx_buf: DmaTxBuf, rx_buf: DmaRxBuf) -> Self {
            Self {
                spi: Some(spi),
                tx_buf: Some(tx_buf),
                rx_buf: Some(rx_buf),
            }
        }
    }

    impl<'d> super::DmaTransfer for EspSpiDma<'d> {
        type Error = esp_hal::spi::Error;

        fn tx_buffer(&mut self) -> &mut [u8] {
            // Access the raw slice. This is zero-copy.
            // We unwrap because the buffer should always be present when idle.
            self.tx_buf.as_mut().unwrap().as_mut_slice()
        }

        fn rx_buffer(&mut self) -> &[u8] {
            self.rx_buf.as_ref().unwrap().as_slice()
        }

        async fn transfer(&mut self, tx_len: usize, rx_len: usize) -> Result<(), Self::Error> {
            // 1. Take ownership of hardware resources
            let spi = self.spi.take().unwrap();
            let mut t_buf = self.tx_buf.take().unwrap();
            let mut r_buf = self.rx_buf.take().unwrap();

            // 2. Configure buffer lengths for this specific transaction
            // TODO: Ensure you don't exceed capacity
            t_buf.set_length(tx_len);
            r_buf.set_length(rx_len);

            // 3. Execute the DMA Transfer
            // This moves the buffers into the driver, and returns them when done.
            let result = spi.transfer(rx_len, r_buf, tx_len, t_buf);

            // 4. Recover ownership / Handle Result
            match result {
                Ok(mut transfer) => {
                    transfer.wait_for_done().await;
                    // TODO: PR to make `wait_for_done` return the same as `wait`?
                    let (spi, (r_buf, t_buf)) = transfer.wait();

                    // Success: Put everything back
                    self.spi = Some(spi);
                    self.tx_buf = Some(t_buf);
                    self.rx_buf = Some(r_buf);
                    Ok(())
                }
                Err((e, ret_spi, ret_r, ret_t)) => {
                    // Error: We still get our buffers back
                    self.spi = Some(ret_spi);
                    self.tx_buf = Some(ret_t);
                    self.rx_buf = Some(ret_r);
                    Err(e)
                }
            }
        }

        async fn transfer_copy(
            &mut self,
            tx_buf: &[u8],
            rx_buf: &mut [u8],
        ) -> Result<(), Self::Error> {
            let spi = self.spi.take().unwrap();
            let t_buf = self.tx_buf.take().unwrap();
            let r_buf = self.rx_buf.take().unwrap();

            let mut spi_dma_bus = spi.with_buffers(r_buf, t_buf);
            let res = spi_dma_bus.transfer_async(rx_buf, tx_buf).await;

            // put back this stuff
            let (spi, r_buf, t_buf) = spi_dma_bus.split();
            self.spi = Some(spi);
            self.rx_buf = Some(r_buf);
            self.tx_buf = Some(t_buf);

            res
        }
    }
}

pub struct SDCard<SPI_DMA, DELAY>
where
    SPI_DMA: DmaTransfer,
    DELAY: Delay,
{
    spi: SPI_DMA,
    delay: DELAY,
}

pub trait Delay {
    type Future: core::future::Future<Output = ()>;
    fn delay_ms(&mut self, ms: u32) -> Self::Future;
}

pub trait DmaTransfer {
    type Error;

    /// Get access to the TX buffer to prepare data (e.g., fill from TCP)
    /// Returns a mutable slice of the underlying DMA memory.
    fn tx_buffer(&mut self) -> &mut [u8];

    /// Get access to the RX buffer to read received data.
    fn rx_buffer(&mut self) -> &[u8];

    /// Execute the transaction.
    /// tx_len: How many bytes from the start of tx_buffer to send.
    /// rx_len: How many bytes to receive into rx_buffer.
    async fn transfer(&mut self, tx_len: usize, rx_len: usize) -> Result<(), Self::Error>;

    /// transfers something by cloning the values
    async fn transfer_copy(&mut self, tx_buf: &[u8], rx_buf: &mut [u8]) -> Result<(), Self::Error>;
}

impl<SPI, DELAY> SDCard<SPI, DELAY>
where
    SPI: DmaTransfer,
    DELAY: Delay,
{
    pub fn init(
        mut spi_bus: impl embedded_hal::spi::SpiDevice<u8>,
        dma: DMA_CH0<'_>,
        delay: DELAY,
    ) {
        todo!()
        // let sd_card = SdCard::new(spi_bus, );
    }

    /// expects an initialized sd card in idle state
    pub fn new(spi_bus: SPI) -> Self {
        todo!();
        // Self { spi: spi_bus }
    }
}
