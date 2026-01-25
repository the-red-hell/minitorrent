use defmt::{debug, info};
use embassy_net::StackResources;
use embassy_time::Duration;
use esp_hal::{peripherals, rng::Rng};
use esp_radio::Controller;

use crate::wifi::{
    EspWifiStack,
    network::{connection, net_task},
};

pub(crate) async fn wifi_setup(
    spawner: embassy_executor::Spawner,
    wifi_peripheral: peripherals::WIFI<'static>,
) -> EspWifiStack {
    let stack = EspWifiStack::initialize(spawner, wifi_peripheral).await;

    stack.0.wait_link_up().await;

    debug!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.0.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        embassy_time::Timer::after(Duration::from_millis(500)).await;
    }

    stack
}

impl EspWifiStack {
    async fn initialize(
        spawner: embassy_executor::Spawner,
        wifi_peripheral: peripherals::WIFI<'static>,
    ) -> Self {
        static ESP_RADIO_CTRL_CELL: static_cell::StaticCell<Controller<'static>> =
            static_cell::StaticCell::new();
        let esp_radio_ctrl = &*ESP_RADIO_CTRL_CELL
            .uninit()
            .write(esp_radio::init().expect("Failed to initialize radio controller"));

        let (wifi_controller, interfaces) =
            esp_radio::wifi::new(esp_radio_ctrl, wifi_peripheral, Default::default())
                .expect("Failed to initialize Wi-Fi controller");

        let config = embassy_net::Config::dhcpv4(Default::default());

        let rng = Rng::new();
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;

        // Init network stack
        static STACK_RESOURCES_CELL: static_cell::StaticCell<StackResources<3>> =
            static_cell::StaticCell::new();
        let (stack, runner) = embassy_net::new(
            interfaces.sta,
            config,
            STACK_RESOURCES_CELL.init(StackResources::<3>::new()),
            seed,
        );
        spawner.spawn(connection(wifi_controller)).ok();
        spawner.spawn(net_task(runner)).ok();

        Self(stack)
    }
}
