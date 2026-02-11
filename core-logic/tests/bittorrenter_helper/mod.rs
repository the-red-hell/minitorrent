use core_logic::BitTorrenter;

use crate::{
    fs_helper::{init_fs_duple, volume_mgr::VolumeMgrDuple},
    wifi_helper::WifiHelper,
};

pub fn init_bittorrenter() -> BitTorrenter<WifiHelper, VolumeMgrDuple> {
    let wifi_helper = WifiHelper;
    let volume_mgr = init_fs_duple();

    BitTorrenter::new(wifi_helper, volume_mgr)
}
