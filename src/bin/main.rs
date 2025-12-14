#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::OnceCell;

use critical_section::Mutex as CriticalMutex;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi::master::{Config, Spi, SpiDmaBus};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_println::{dbg, println};
use panic_rtt_target as _;

use sdmmc::SD;
use sdmmc::bus::spi::{Bus, Transfer};
use sdmmc::delay::Delay as DelayTrait;

extern crate alloc;

static RTC_CLOCK: CriticalMutex<OnceCell<Rtc>> = CriticalMutex::new(OnceCell::new());

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    // rtt_target::rtt_init_defmt!();
    println!("hi");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // info!("Embassy initialized!");
    println!("Embassy initialized!");

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    // TODO: Spawn some tasks
    let _ = spawner;

    // yo
    // DMA
    let dma_channel = peripherals.DMA_CH0;
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = esp_hal::dma_buffers!(16000);

    let dma_rx_buf = esp_hal::dma::DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();

    let dma_tx_buf = esp_hal::dma::DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // SPI
    let mut spi_bus = Spi::new(
        peripherals.SPI2,
        Config::default().with_frequency(Rate::from_khz(250)),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_miso(peripherals.GPIO2)
    .with_mosi(peripherals.GPIO7)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    let idk = spi_bus.write_async(&[0]).await;
    dbg!(idk.unwrap());

    println!("spi bus initialized");

    let cs = Output::new(
        peripherals.GPIO10,
        esp_hal::gpio::Level::High,
        OutputConfig::default(),
    );

    // static SPI_BUS: StaticCell<Mutex<NoopRawMutex, SpiDmaBus<esp_hal::Async>>> = StaticCell::new();
    // let spi_bus = SPI_BUS.init(Mutex::new(spi_bus));

    // let spi = SpiDevice::new(spi_bus, cs);

    // SD
    critical_section::with(|cs| {
        if RTC_CLOCK
            .borrow(cs)
            .set(Rtc::new(peripherals.LPWR))
            .is_err()
        {
            panic!("should not be initialized");
        }
    });
    println!("RTC Clock initialized");
    let mut bus = Bus::new(SPI(spi_bus), cs, SystemClock);
    let card = bus.init(Delay).await.unwrap();
    println!("wrote to card");
    let sd = SD::init(bus, card).await.unwrap();
    println!("read csd");
    println!("{}", sd.num_blocks().device_size());

    // let sd_card = SdCard::new(spi, Delay::new());

    loop {
        println!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}

struct Delay;

impl DelayTrait for Delay {
    type Future = Timer;

    fn delay_ms(&mut self, ms: u32) -> Self::Future {
        println!("waiting for {ms}");
        Timer::after_millis(ms as u64)
    }
}

struct SPI<'a>(SpiDmaBus<'a, esp_hal::Async>);

impl<'a> Transfer for SPI<'a> {
    type Error = esp_hal::spi::Error;

    fn transfer(
        &mut self,
        tx: &[u8],
        rx: &mut [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> {
        println!("transferring some bytes: {} {}", tx.len(), rx.len());
        self.0.transfer_async(rx, tx)
    }
}

struct SystemClock;
impl embedded_timers::clock::Clock for SystemClock {
    type Instant = embedded_timers::instant::Instant64<1000>;

    fn now(&self) -> Self::Instant {
        // SAFETY: we inialize it before we create a clock.
        embedded_timers::instant::Instant64::new(critical_section::with(|cs| {
            dbg!(RTC_CLOCK.borrow(cs).get().unwrap().current_time_us())
        }))
    }
}
