#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embedded_sdmmc::VolumeManager;
use panic_rtt_target as _;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let mut bittorrenter = esp_app::setup::setup(spawner).await;

    let mut buf = [0u8; 1024 * 10];
    let file_length = bittorrenter
        .fs()
        .put_torrent_into_buf(&mut buf)
        .await
        .unwrap();
    info!("WE GOT THE FILE WITH LENGTH: {:?}", file_length);

    let torrent = defmt::unwrap!(core_logic::core::metainfo::MetaInfoFile::parse(
        &buf[..file_length]
    ));

    info!("WE GOT THE TORRENT WITH: {:?}", torrent);

    let mut rx_buf = [0u8; 1024];
    let res = bittorrenter.into_downloader(&torrent, &mut rx_buf).await;
    match res {
        Ok(mut downloader) => {
            info!("WE GOT A TRACKER RESPONSE: {:?}", downloader.get_peers());

            match downloader.download().await {
                Ok(_) => info!("DOWNLOAD COMPLETED SUCCESSFULLY"),
                Err(e) => info!("DOWNLOAD FAILED WITH ERROR: {:?}", e),
            }

            VolumeManager::close_file(
                downloader.fs.get_volume_mgr(),
                downloader.fs.get_open_file().unwrap(),
            )
            .unwrap();
        }
        Err(e) => {
            info!("WE GOT AN ERROR FROM THE TRACKER {}", e);
        }
    }

    #[allow(clippy::empty_loop)]
    loop {}
}
