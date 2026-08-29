#[derive(Debug, Clone)]
pub enum SystemEvent {
    WifiChanged(bool),
    WifiNetworksChanged(Vec<WifiNetworkInfo>),
    BluetoothChanged(bool),
    BluetoothDevicesChanged(Vec<BluetoothDeviceInfo>),
    ApiHealthChanged(bool),
    TimeChanged(String),
    SpotifyTracksLoaded(Vec<crate::spotify::SpotifyTrack>),
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
pub enum NetworkCommand {
    SetWifiEnabled(bool),
    ScanWifi,
    SetWifiCredentials { ssid: String, password: String },
    SetBluetoothEnabled(bool),
    ScanBluetooth,
    SetTimezoneOffset(i32),
    MusicCommand(String),
}
