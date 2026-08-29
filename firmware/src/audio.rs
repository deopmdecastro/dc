//! Audio I2S: TX funcional para o amplificador e RX para captura do microfone.
use anyhow::{anyhow, Result};
use crate::pinout::pins::{I2S_BCLK, I2S_DIN, I2S_DOUT, I2S_WS};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static I2S_INITIALIZED: AtomicBool = AtomicBool::new(false);
static I2S_TX_MODE: AtomicBool = AtomicBool::new(false);
static I2S_LISTENING: AtomicBool = AtomicBool::new(false);
static AUDIO_CALLBACK: Mutex<Option<Box<dyn Fn(&[i16]) + Send>>> = Mutex::new(None);

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: u8 = 1;
pub const BITS: u8 = 16;
pub const BUFFER_SIZE: usize = 1024;

/// Emite um beep curto no DAC/amp I2S para validar o caminho de audio.
pub fn play_test_tone(tone: u8) {
    std::thread::spawn(move || {
        if let Err(e) = write_tone(tone) { log::warn!("Audio I2S: teste falhou: {e:?}"); }
    });
}

unsafe fn init_i2s_tx() -> Result<()> {
    let port = esp_idf_sys::i2s_port_t_I2S_NUM_0;
    // Se ja esta inicializado em modo RX, desinstalar primeiro
    if I2S_INITIALIZED.load(Ordering::SeqCst) && !I2S_TX_MODE.load(Ordering::SeqCst) {
        esp_idf_sys::i2s_driver_uninstall(port);
        I2S_INITIALIZED.store(false, Ordering::SeqCst);
    }
    if !I2S_INITIALIZED.load(Ordering::SeqCst) {
        let mut cfg: esp_idf_sys::i2s_config_t = core::mem::zeroed();
        cfg.mode = (esp_idf_sys::i2s_mode_t_I2S_MODE_MASTER | esp_idf_sys::i2s_mode_t_I2S_MODE_TX) as u32;
        cfg.sample_rate = SAMPLE_RATE;
        cfg.bits_per_sample = esp_idf_sys::i2s_bits_per_sample_t_I2S_BITS_PER_SAMPLE_16BIT;
        cfg.channel_format = esp_idf_sys::i2s_channel_fmt_t_I2S_CHANNEL_FMT_ONLY_LEFT;
        cfg.communication_format = esp_idf_sys::i2s_comm_format_t_I2S_COMM_FORMAT_STAND_I2S;
        cfg.intr_alloc_flags = 0;
        cfg.__bindgen_anon_1.dma_buf_count = 4;
        cfg.__bindgen_anon_2.dma_buf_len = 256;
        cfg.use_apll = false;
        cfg.tx_desc_auto_clear = true;
        cfg.fixed_mclk = 0;
        let pins = esp_idf_sys::i2s_pin_config_t {
            bck_io_num: I2S_BCLK as i32, ws_io_num: I2S_WS as i32,
            data_out_num: I2S_DOUT as i32, data_in_num: -1,
            mck_io_num: -1,
        };
        let err = esp_idf_sys::i2s_driver_install(port, &cfg, 0, core::ptr::null_mut());
        if err != 0 && err != esp_idf_sys::ESP_ERR_INVALID_STATE { return Err(anyhow!("i2s_driver_install TX: {err}")); }
        let err = esp_idf_sys::i2s_set_pin(port, &pins);
        if err != 0 { return Err(anyhow!("i2s_set_pin TX: {err}")); }
        I2S_INITIALIZED.store(true, Ordering::SeqCst);
        I2S_TX_MODE.store(true, Ordering::SeqCst);
    }
    Ok(())
}

unsafe fn init_i2s_rx() -> Result<()> {
    let port = esp_idf_sys::i2s_port_t_I2S_NUM_0;
    // Se ja esta inicializado em modo TX, desinstalar primeiro
    if I2S_INITIALIZED.load(Ordering::SeqCst) && I2S_TX_MODE.load(Ordering::SeqCst) {
        esp_idf_sys::i2s_driver_uninstall(port);
        I2S_INITIALIZED.store(false, Ordering::SeqCst);
    }
    if !I2S_INITIALIZED.load(Ordering::SeqCst) {
        let mut cfg: esp_idf_sys::i2s_config_t = core::mem::zeroed();
        cfg.mode = (esp_idf_sys::i2s_mode_t_I2S_MODE_MASTER | esp_idf_sys::i2s_mode_t_I2S_MODE_RX) as u32;
        cfg.sample_rate = SAMPLE_RATE;
        cfg.bits_per_sample = esp_idf_sys::i2s_bits_per_sample_t_I2S_BITS_PER_SAMPLE_16BIT;
        cfg.channel_format = esp_idf_sys::i2s_channel_fmt_t_I2S_CHANNEL_FMT_ONLY_LEFT;
        cfg.communication_format = esp_idf_sys::i2s_comm_format_t_I2S_COMM_FORMAT_STAND_I2S;
        cfg.intr_alloc_flags = 0;
        cfg.__bindgen_anon_1.dma_buf_count = 4;
        cfg.__bindgen_anon_2.dma_buf_len = 256;
        cfg.use_apll = false;
        cfg.tx_desc_auto_clear = false;
        cfg.fixed_mclk = 0;
        let pins = esp_idf_sys::i2s_pin_config_t {
            bck_io_num: I2S_BCLK as i32, ws_io_num: I2S_WS as i32,
            data_out_num: -1, data_in_num: I2S_DIN as i32,
            mck_io_num: -1,
        };
        let err = esp_idf_sys::i2s_driver_install(port, &cfg, 0, core::ptr::null_mut());
        if err != 0 && err != esp_idf_sys::ESP_ERR_INVALID_STATE { return Err(anyhow!("i2s_driver_install RX: {err}")); }
        let err = esp_idf_sys::i2s_set_pin(port, &pins);
        if err != 0 { return Err(anyhow!("i2s_set_pin RX: {err}")); }
        I2S_INITIALIZED.store(true, Ordering::SeqCst);
        I2S_TX_MODE.store(false, Ordering::SeqCst);
    }
    Ok(())
}

fn write_tone(tone: u8) -> Result<()> {
    unsafe {
        init_i2s_tx()?;
        let port = esp_idf_sys::i2s_port_t_I2S_NUM_0;
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

/// Inicia a captura de audio do microfone.
pub fn start_listening(callback: Box<dyn Fn(&[i16]) + Send>) -> Result<()> {
    unsafe { init_i2s_rx()?; }
    {
        let mut cb = AUDIO_CALLBACK.lock().map_err(|_| anyhow!("lock poisoned"))?;
        *cb = Some(callback);
    }
    I2S_LISTENING.store(true, Ordering::SeqCst);
    std::thread::spawn(move || {
        log::info!("Audio I2S: captura iniciada");
        let mut buffer = vec![0i16; BUFFER_SIZE];
        while I2S_LISTENING.load(Ordering::SeqCst) {
            unsafe {
                let port = esp_idf_sys::i2s_port_t_I2S_NUM_0;
                let mut read = 0usize;
                let buf_bytes = (buffer.len() * 2) as usize;
                let err = esp_idf_sys::i2s_read(port, buffer.as_mut_ptr() as *mut core::ffi::c_void, buf_bytes, &mut read, 100);
                if err == 0 && read > 0 {
                    let samples = read / 2;
                    if let Ok(cb) = AUDIO_CALLBACK.lock() {
                        if let Some(ref f) = *cb {
                            f(&buffer[..samples]);
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::info!("Audio I2S: captura parada");
    });
    Ok(())
}

/// Para a captura de audio.
pub fn stop_listening() {
    I2S_LISTENING.store(false, Ordering::SeqCst);
}

/// Calcula o nivel RMS do audio (0.0 a 1.0).
pub fn calculate_rms(samples: &[i16]) -> f32 {
    if samples.is_empty() { return 0.0; }
    let sum_sq: f64 = samples.iter().map(|s| { let v = *s as f64; v * v }).sum();
    let rms = (sum_sq / samples.len() as f64).sqrt();
    (rms / 32768.0).min(1.0) as f32
}

/// Inicializa o caminho de audio.
pub fn spawn_audio_task<L>(_on_level: L) -> Result<()>
where L: Fn(f32) + Send + 'static {
    log::info!("Audio I2S pronto ({} Hz, BCLK={}, WS={}, DIN={}, DOUT={})", SAMPLE_RATE, I2S_BCLK, I2S_WS, I2S_DIN, I2S_DOUT);
    Ok(())
}

/// Reproduz um ficheiro WAV/PCM local atraves do I2S.
pub fn play_file(path: &str) -> Result<()> {
    use std::fs;
    let data = fs::read(path)?;
    let pcm = if data.len() >= 44 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        &data[44..]
    } else {
        return Err(anyhow!("Formato nao suportado: use WAV PCM 16-bit mono"));
    };
    write_samples(bytes_to_pcm(pcm))
}

fn bytes_to_pcm(data: &[u8]) -> Vec<i16> {
    let mut samples = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample);
    }
    samples
}

fn write_samples(samples: Vec<i16>) -> Result<()> {
    std::thread::spawn(move || {
        unsafe {
            let port = esp_idf_sys::i2s_port_t_I2S_NUM_0;
            if let Err(e) = init_i2s_tx() {
                log::warn!("I2S TX nao inicializado: {e:?}");
                return;
            }
            let mut written = 0usize;
            let _ = esp_idf_sys::i2s_write(
                port,
                samples.as_ptr() as *const core::ffi::c_void,
                (samples.len() * 2) as usize,
                &mut written,
                1000,
            );
        }
    });
    Ok(())
}
