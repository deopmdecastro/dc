//! Audio I2S: TX funcional para o amplificador e ponto de extensao para RX/STT.
use anyhow::{anyhow, Result};
use crate::pinout::{I2S_BCLK, I2S_DIN, I2S_DOUT, I2S_WS};

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: u8 = 1;
pub const BITS: u8 = 16;

/// Emite um beep curto no DAC/amp I2S para validar o caminho de audio.
pub fn play_test_tone(tone: u8) {
    std::thread::spawn(move || {
        if let Err(e) = write_tone(tone) { log::warn!("Audio I2S: teste falhou: {e:?}"); }
    });
}

fn write_tone(tone: u8) -> Result<()> {
    unsafe {
        let mut cfg: esp_idf_sys::i2s_config_t = core::mem::zeroed();
        cfg.mode = (esp_idf_sys::i2s_mode_t_I2S_MODE_MASTER | esp_idf_sys::i2s_mode_t_I2S_MODE_TX) as i32;
        cfg.sample_rate = SAMPLE_RATE as i32;
        cfg.bits_per_sample = esp_idf_sys::i2s_bits_per_sample_t_I2S_BITS_PER_SAMPLE_16BIT;
        cfg.channel_format = esp_idf_sys::i2s_channel_fmt_t_I2S_CHANNEL_FMT_ONLY_LEFT;
        cfg.communication_format = esp_idf_sys::i2s_comm_format_t_I2S_COMM_FORMAT_STAND_I2S;
        cfg.intr_alloc_flags = 0;
        cfg.dma_buf_count = 4;
        cfg.dma_buf_len = 256;
        cfg.use_apll = false;
        cfg.tx_desc_auto_clear = true;
        cfg.fixed_mclk = 0;
        let pins = esp_idf_sys::i2s_pin_config_t {
            bck_io_num: I2S_BCLK as i32, ws_io_num: I2S_WS as i32,
            data_out_num: I2S_DOUT as i32, data_in_num: I2S_DIN as i32,
        };
        let port = esp_idf_sys::i2s_port_t_I2S_NUM_0;
        let err = esp_idf_sys::i2s_driver_install(port, &cfg, 0, core::ptr::null_mut());
        if err != 0 && err != esp_idf_sys::ESP_ERR_INVALID_STATE { return Err(anyhow!("i2s_driver_install: {err}")); }
        let err = esp_idf_sys::i2s_set_pin(port, &pins);
        if err != 0 { return Err(anyhow!("i2s_set_pin: {err}")); }
        let freq = 440 + (tone as i32 * 110);
        let samples = (SAMPLE_RATE / 5) as usize;
        let mut pcm = vec![0i16; samples];
        for (i, sample) in pcm.iter_mut().enumerate() {
            let phase = (i as f32 * freq as f32 * core::f32::consts::TAU / SAMPLE_RATE as f32).sin();
            *sample = (phase * 7000.0) as i16;
        }
        let mut written = 0usize;
        let err = esp_idf_sys::i2s_write(port, pcm.as_ptr() as *const core::ffi::c_void, (pcm.len()*2) as usize, &mut written, 1000);
        if err != 0 { return Err(anyhow!("i2s_write: {err}")); }
        let _ = esp_idf_sys::i2s_zero_dma_buffer(port);
        log::info!("Audio I2S: tom de teste emitido em GPIO {} ({} bytes)", I2S_DOUT, written);
    }
    Ok(())
}

/// Inicializa o caminho de audio. O RX/STT continua sendo ativado quando a captura for ligada.
pub fn spawn_audio_task<L, P>(_on_level: L, _on_pcm: P) -> Result<()>
where L: Fn(f32) + Send + 'static, P: Fn(&[i16]) + Send + 'static {
    log::info!("Audio I2S TX pronto ({} Hz, BCLK={}, WS={}, DOUT={})", SAMPLE_RATE, I2S_BCLK, I2S_WS, I2S_DOUT);
    Ok(())
}
