use defmt::{debug, error, info};
use embassy_net::Runner;
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{Interface, WifiController, sta::StationConfig};

// my mobile hotspot, don't worry
const SSID: &str = "paul";
const PASSWORD: &str = "00000000";

/// connects the esp to wifi and also makes sure it stays connected
#[embassy_executor::task]
pub(super) async fn wifi_connection_task(mut controller: WifiController<'static>) {
    debug!("start connection task");

    loop {
        if controller.is_connected() {
            controller.wait_for_disconnect_async().await.unwrap(); // TODO
            Timer::after(Duration::from_millis(5000)).await;
        }
        debug!("Starting wifi");
        let client_config = esp_radio::wifi::Config::Station(
            StationConfig::default()
                .with_ssid(SSID)
                .with_password(PASSWORD.into()),
        );
        controller
            .set_config(&client_config)
            .expect("Failed to set WiFi configuration");
        debug!("Wifi started!");
        debug!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                error!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
pub(super) async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}
