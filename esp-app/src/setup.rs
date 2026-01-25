use core_logic::BitTorrenter;
use defmt::info;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};

use crate::{
    fs::{EspFileSystem, sd_card},
    wifi::{self, EspWifiStack},
};

pub async fn setup(
    spawner: embassy_executor::Spawner,
) -> BitTorrenter<EspWifiStack, EspFileSystem> {
    // generator version: 1.0.1

    rtt_target::rtt_init_defmt!();
    info!("hi");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    // WIFI
    let wifi = wifi::setup::wifi_setup(spawner, peripherals.WIFI).await;

    // SD CARD
    let fs_init = sd_card::SPIInitializer::new(
        peripherals.GPIO4,
        peripherals.GPIO5,
        peripherals.GPIO6,
        peripherals.GPIO7,
    );
    let fs = EspFileSystem::setup(fs_init, peripherals.SPI2)
        .await
        .unwrap();

    info!("Done initializing.");

    BitTorrenter::new(wifi, fs)
}
