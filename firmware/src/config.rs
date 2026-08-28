//! Configuracao persistente do DC OS em NVS.

use anyhow::Result;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

const NAMESPACE: &str = "dc_os";
const KEY_PASSCODE: &str = "passcode";
const KEY_WIFI_ON: &str = "wifi_on";
const KEY_WIFI_SSID: &str = "wifi_ssid";
const KEY_WIFI_PASS: &str = "wifi_pass";
const KEY_BT_ON: &str = "bt_on";
const KEY_API_HEALTH: &str = "api_health";
const KEY_REGION: &str = "region";
const KEY_LANGUAGE: &str = "language";
const KEY_ALARM_ON: &str = "alarm_on";
const KEY_ALARM_HOUR: &str = "alarm_hour";
const KEY_ALARM_MINUTE: &str = "alarm_minute";
const KEY_VOLUME: &str = "volume";
const KEY_BRIGHTNESS: &str = "brightness";

const MAX_PASSCODE: usize = 16;
const MAX_SSID: usize = 64;
const MAX_PASSWORD: usize = 96;
const MAX_URL: usize = 160;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub passcode: Option<String>,
    pub wifi_enabled: bool,
    pub wifi_ssid: String,
    pub wifi_password: String,
    pub bluetooth_enabled: bool,
    pub api_health_url: String,
    pub region_index: u8,
    pub language_index: u8,
    pub alarm_enabled: bool,
    pub alarm_hour: u8,
    pub alarm_minute: u8,
    pub volume: u8,
    pub brightness: u8,
}

pub struct ConfigStore {
    nvs: EspNvs<NvsDefault>,
}

impl ConfigStore {
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self> {
        Ok(Self {
            nvs: EspNvs::new(partition, NAMESPACE, true)?,
        })
    }

    pub fn load(&self) -> AppConfig {
        let default_ssid = option_env!("DC_WIFI_SSID").unwrap_or("DC_Network");
        let default_pass = option_env!("DC_WIFI_PASS").unwrap_or("");
        let default_api = option_env!("DC_CORE_HTTP").unwrap_or("http://192.168.1.50:8080/health");

        AppConfig {
            passcode: self.read_string(KEY_PASSCODE, MAX_PASSCODE),
            wifi_enabled: self.read_bool(KEY_WIFI_ON).unwrap_or(true),
            wifi_ssid: self
                .read_string(KEY_WIFI_SSID, MAX_SSID)
                .unwrap_or_else(|| default_ssid.to_owned()),
            wifi_password: self
                .read_string(KEY_WIFI_PASS, MAX_PASSWORD)
                .unwrap_or_else(|| default_pass.to_owned()),
            bluetooth_enabled: self.read_bool(KEY_BT_ON).unwrap_or(false),
            api_health_url: self
                .read_string(KEY_API_HEALTH, MAX_URL)
                .unwrap_or_else(|| default_api.to_owned()),
            region_index: self.read_u8(KEY_REGION).unwrap_or(0).min(4),
            language_index: self.read_u8(KEY_LANGUAGE).unwrap_or(0).min(4),
            alarm_enabled: self.read_bool(KEY_ALARM_ON).unwrap_or(false),
            alarm_hour: self.read_u8(KEY_ALARM_HOUR).unwrap_or(7).min(23),
            alarm_minute: self.read_u8(KEY_ALARM_MINUTE).unwrap_or(0).min(59),
            volume: self.read_u8(KEY_VOLUME).unwrap_or(75).min(100),
            brightness: self.read_u8(KEY_BRIGHTNESS).unwrap_or(60).min(100),
        }
    }

    pub fn save_passcode(&self, passcode: &str) -> Result<()> {
        if passcode.len() <= MAX_PASSCODE {
            self.nvs.set_str(KEY_PASSCODE, passcode)?;
        }
        Ok(())
    }

    pub fn save_wifi_enabled(&self, enabled: bool) -> Result<()> {
        self.write_bool(KEY_WIFI_ON, enabled)
    }

    pub fn save_wifi_credentials(&self, ssid: &str, password: &str) -> Result<()> {
        if ssid.len() <= MAX_SSID {
            self.nvs.set_str(KEY_WIFI_SSID, ssid)?;
        }
        if password.len() <= MAX_PASSWORD {
            self.nvs.set_str(KEY_WIFI_PASS, password)?;
        }
        Ok(())
    }

    pub fn save_bluetooth_enabled(&self, enabled: bool) -> Result<()> {
        self.write_bool(KEY_BT_ON, enabled)
    }

    pub fn save_locale(&self, region_index: u8, language_index: u8) -> Result<()> {
        self.nvs.set_u8(KEY_REGION, region_index.min(4))?;
        self.nvs.set_u8(KEY_LANGUAGE, language_index.min(4))?;
        Ok(())
    }

    pub fn save_alarm(&self, enabled: bool, hour: u8, minute: u8) -> Result<()> {
        self.write_bool(KEY_ALARM_ON, enabled)?;
        self.nvs.set_u8(KEY_ALARM_HOUR, hour.min(23))?;
        self.nvs.set_u8(KEY_ALARM_MINUTE, minute.min(59))?;
        Ok(())
    }

    pub fn save_volume(&self, value: u8) -> Result<()> {
        self.nvs.set_u8(KEY_VOLUME, value.min(100))?;
        Ok(())
    }

    pub fn save_brightness(&self, value: u8) -> Result<()> {
        self.nvs.set_u8(KEY_BRIGHTNESS, value.min(100))?;
        Ok(())
    }

    fn read_string(&self, key: &str, max_len: usize) -> Option<String> {
        let mut buf = vec![0_u8; max_len + 1];
        match self.nvs.get_str(key, &mut buf) {
            Ok(Some(value)) if !value.is_empty() => Some(value.to_owned()),
            Ok(_) => None,
            Err(e) => {
                log::warn!("NVS: falha ao ler {key}: {e:?}");
                None
            }
        }
    }

    fn read_bool(&self, key: &str) -> Option<bool> {
        self.read_u8(key).map(|value| value != 0)
    }

    fn read_u8(&self, key: &str) -> Option<u8> {
        match self.nvs.get_u8(key) {
            Ok(Some(value)) => Some(value),
            Ok(None) => None,
            Err(e) => {
                log::warn!("NVS: falha ao ler {key}: {e:?}");
                None
            }
        }
    }

    fn write_bool(&self, key: &str, enabled: bool) -> Result<()> {
        self.nvs.set_u8(key, if enabled { 1 } else { 0 })?;
        Ok(())
    }
}
