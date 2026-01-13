#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::OnceCell;

use critical_section::Mutex as CriticalMutex;
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::timg::TimerGroup;
use minitorrent::fs::FileSystem;
use minitorrent::fs::sd_card::FileSystemInitializer;
use panic_rtt_target as _;

use crate::get_torrents::get_torrent;

mod get_torrents;

extern crate alloc;

static _RTC_CLOCK: CriticalMutex<OnceCell<Rtc>> = CriticalMutex::new(OnceCell::new());

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    rtt_target::rtt_init_defmt!();
    defmt::info!("hi");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    defmt::info!("Embassy initialized!");

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    // TODO: Spawn some tasks
    let _ = spawner;

    // yo

    let fs_init = FileSystemInitializer::new(
        peripherals.GPIO4,
        peripherals.GPIO5,
        peripherals.GPIO6,
        peripherals.GPIO7,
    );
    let _fs = FileSystem::initialize(fs_init, peripherals.SPI2)
        .await
        .unwrap();

    let file = get_torrent().await.unwrap();
    defmt::warn!("WE GOT THE FILE WITH: {:?}", file.as_slice());
    // let volume_mgr = sdcard.to_volume_mgr();
    // let volume = volume_mgr.get_volume();
    // let root_dir = volume.open_root_dir();

    loop {}
}
