#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::{OnceCell, RefCell};

use critical_section::Mutex as CriticalMutex;
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer};
use embedded_hal::spi::SpiBus;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::SdCard;
// use embedded_sdmmc::FatVolume;
use embedded_io_async::{Read, Seek, Write};
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
        Config::default().with_frequency(Rate::from_khz(250)), //  max: 80MHz
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_miso(peripherals.GPIO2)
    .with_mosi(peripherals.GPIO7)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    println!("spi bus initialized");

    let mut cs = Output::new(
        peripherals.GPIO10,
        esp_hal::gpio::Level::High,
        OutputConfig::default(),
    );

    // Sd cards need to be clocked with a at least 74 cycles on their spi clock without the cs enabled,
    // sd_init is a helper function that does this for us.
    loop {
        match sdspi::sd_init(&mut spi_bus, &mut cs).await {
            Ok(_) => break,
            Err(e) => {
                println!("Sd init error: {:?}", e);
                embassy_time::Timer::after_millis(10).await;
            }
        }
    }

    let spid = ExclusiveDevice::new(spi_bus, cs, embassy_time::Delay).unwrap();
    let mut sd = sdspi::SdSpi::<_, _, aligned::A1>::new(spid, embassy_time::Delay);

    loop {
        // Initialize the card
        if sd.init().await.is_ok() {
            // Increase the speed up to the SD max of 25mhz
            let _ = sd
                .spi()
                .bus_mut()
                .apply_config(&Config::default().with_frequency(Rate::from_mhz(25)));
            println!("Initialization complete!");

            break;
        }
        println!("Failed to init card, retrying...");
        embassy_time::Delay.delay_ns(5000u32).await;
    }

    let inner = block_device_adapters::BufStream::<_, 512>::new(sd);

    // async {
    let fs = embedded_fatfs::FileSystem::new(inner, embedded_fatfs::FsOptions::new())
        .await
        .unwrap();
    {
        let mut f = fs.root_dir().create_file("test.log").await.unwrap();
        let hello = b"Hello world!";
        println!("Writing to file...");
        f.write_all(hello).await.unwrap();
        f.flush().await.unwrap();

        let mut buf = [0u8; 12];
        f.rewind().await.unwrap();
        f.read_exact(&mut buf[..]).await.unwrap();
        println!(
            "Read from file: {}",
            core::str::from_utf8(&buf[..]).unwrap()
        );
    }
    fs.unmount().await.unwrap();

    // Ok::<(), embedded_fatfs::Error<BufStreamError<sdspi::Error>>>(())
    // }
    // .await
    // .expect("Filesystem tests failed!");

    loop {}

    // // Using a CriticalSectionRawMutex and RefCell for the shared blocking SPI bus
    // static SPI_BUS_INST: static_cell::StaticCell<
    //     embassy_sync::blocking_mutex::Mutex<
    //         CriticalSectionRawMutex,
    //         core::cell::RefCell<Spi<esp_hal::Blocking>>,
    //     >,
    // > = static_cell::StaticCell::new();

    // let spi_bus_shared_ref = SPI_BUS_INST.init(embassy_sync::blocking_mutex::Mutex::new(
    //     core::cell::RefCell::new(spi_bus),
    // ));
    // let spi =
    //     embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice::new(spi_bus_shared_ref, cs);

    // // SD
    // critical_section::with(|cs| {
    //     if RTC_CLOCK
    //         .borrow(cs)
    //         .set(Rtc::new(peripherals.LPWR))
    //         .is_err()
    //     {
    //         panic!("should not be initialized");
    //     }
    // });

    // let sd_card = SdCard::new(spi, esp_hal::delay::Delay::new());
    // dbg!(sd_card.num_bytes());

    // println!("RTC Clock initialized");
    // let mut bus = Bus::new(spi_bus, cs, SystemClock);
    // let card = dbg!(bus.init(Delay).await).unwrap();
    // println!("wrote to card");
    // let sd = SD::init(bus, card).await.inspect_err(|e| {
    //     dbg!(e);
    // });
    // println!("read csd");
    // println!("{}", sd.unwrap().num_blocks().device_size());

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

// struct SPI<'a>(Spi<'a, esp_hal::Async>);

// impl<'a> Transfer for SPI<'a> {
//     type Error = esp_hal::spi::Error;

//     async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), Self::Error> {
//         println!("transferring {:?}, receiving {:?}", tx, rx);
//         match (!tx.is_empty(), !rx.is_empty()) {
//             (true, true) => SpiBus::transfer(&mut self.0, rx, tx),
//             (true, false) => self.0.read(rx),
//             (false, true) => self.0.write(tx).await,
//             _ => unreachable!(),
//         }
//     }
// }

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
