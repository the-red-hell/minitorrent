use defmt::info;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};

use crate::{
    fs::{FileSystem, sd_card},
    wifi,
};

pub async fn setup(spawner: embassy_executor::Spawner) {
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
    wifi::setup::wifi_setup(spawner, peripherals.WIFI).await;

    // SD CARD
    let fs_init = sd_card::SPIInitializer::new(
        peripherals.GPIO4,
        peripherals.GPIO5,
        peripherals.GPIO6,
        peripherals.GPIO7,
    );
    FileSystem::setup(fs_init, peripherals.SPI2).await.unwrap();

    info!("Done initializing.");

    // let file = get_torrent().await.unwrap();
    // warn!("WE GOT THE FILE WITH: {:?}", file.as_slice());
    // let volume_mgr = sdcard.to_volume_mgr();
    // let volume = volume_mgr.get_volume();
    // let root_dir = volume.open_root_dir();
}
