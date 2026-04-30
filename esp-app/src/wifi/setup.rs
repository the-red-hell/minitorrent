use defmt::{debug, info};
use embassy_net::StackResources;
use embassy_time::Duration;
use esp_hal::{peripherals, rng::Rng};

use crate::wifi::{
    EspWifi,
    network::{net_task, wifi_connection_task},
};

pub(crate) async fn wifi_setup(
    spawner: embassy_executor::Spawner,
    wifi_peripheral: peripherals::WIFI<'static>,
) -> EspWifi {
    let wifi = EspWifi::initialize(spawner, wifi_peripheral).await;

    wifi.stack().wait_link_up().await;

    debug!("Waiting to get IP address...");
    loop {
        if let Some(config) = wifi.stack().config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        embassy_time::Timer::after(Duration::from_millis(500)).await;
    }

    wifi
}

impl EspWifi {
    async fn initialize(
        spawner: embassy_executor::Spawner,
        wifi_peripheral: peripherals::WIFI<'static>,
    ) -> Self {
        let (controller, interfaces) =
            esp_radio::wifi::new(wifi_peripheral, Default::default()).unwrap();

        let config = embassy_net::Config::dhcpv4(Default::default());

        let rng = Rng::new();
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;

        // Init network stack
        static STACK_RESOURCES_CELL: static_cell::StaticCell<StackResources<3>> =
            static_cell::StaticCell::new();
        let (stack, runner) = embassy_net::new(
            interfaces.station,
            config,
            STACK_RESOURCES_CELL.init(StackResources::<3>::new()),
            seed,
        );
        spawner.spawn(wifi_connection_task(controller).expect("// TODO"));
        spawner.spawn(net_task(runner).expect("// TODO"));

        Self::new(stack)
    }
}
