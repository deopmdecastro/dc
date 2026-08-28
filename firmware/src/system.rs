#[derive(Debug, Clone)]
pub enum SystemEvent {
    WifiChanged(bool),
    BluetoothChanged(bool),
    ApiHealthChanged(bool),
    TimeChanged(String),
    SpotifyTracksLoaded(Vec<crate::spotify::SpotifyTrack>),
}

#[derive(Debug, Clone)]
pub enum NetworkCommand {
    SetWifiEnabled(bool),
    SetWifiCredentials { ssid: String, password: String },
    SetTimezoneOffset(i32),
    MusicCommand(String),
}
