#[derive(Debug, Clone)]
pub enum SystemEvent {
    WifiChanged(bool),
    WifiNetworksChanged(Vec<WifiNetworkInfo>),
    BluetoothChanged(bool),
    BluetoothDevicesChanged(Vec<BluetoothDeviceInfo>),
    ApiHealthChanged(bool),
    TimeChanged(String),
    SpotifyTracksLoaded(Vec<crate::spotify::SpotifyTrack>),
    WeatherChanged(WeatherInfo),
}

#[derive(Debug, Clone)]
pub struct WifiNetworkInfo {
    pub ssid: String,
    pub secured: bool,
    pub connected: bool,
    pub signal_strength: i8,
}

#[derive(Debug, Clone)]
pub struct BluetoothDeviceInfo {
    pub address: String,
    pub rssi: i32,
}

#[derive(Debug, Clone)]
pub struct WeatherInfo {
    pub city: String,
    pub temperature_c: i32,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub enum NetworkCommand {
    SetWifiEnabled(bool),
    ScanWifi,
    SetWifiCredentials {
        ssid: String,
        password: String,
    },
    SetBluetoothEnabled(bool),
    ScanBluetooth,
    SetLocale {
        region_index: u8,
        timezone_offset_secs: i32,
    },
    MusicCommand(String),
}
