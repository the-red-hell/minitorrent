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
use esp_hal::rtc_cntl::Rtc;
use panic_rtt_target as _;

extern crate alloc;

static _RTC_CLOCK: CriticalMutex<OnceCell<Rtc>> = CriticalMutex::new(OnceCell::new());

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    esp_app::setup::setup(spawner).await;

    let file = esp_app::torrent_retrieval::get_torrent_from_file()
        .await
        .unwrap();
    info!("WE GOT THE FILE WITH: {:?}", file.as_slice());

    loop {}
}
