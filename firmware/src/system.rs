#[derive(Debug, Clone)]
pub enum SystemEvent {
    WifiChanged(bool),
    BluetoothChanged(bool),
    ApiHealthChanged(bool),
    TimeChanged(String),
}

#[derive(Debug, Clone)]
pub enum NetworkCommand {
    SetWifiEnabled(bool),
    SetWifiCredentials { ssid: String, password: String },
}
