//! Task de áudio: captura MEMS (I2S RX) → WebSocket, e reprodução
//! (WS ← backend) → I2S TX no alto-falante.

use anyhow::Result;

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS:    u8  = 1;
pub const BITS:        u8  = 16;

/// Spawna a task de áudio full-duplex. Recebe closures para:
///  - `on_level(f32)`   → atualiza a onda visual (0..1)
///  - `on_pcm(&[i16])`  → streaming para o backend (STT/Wake-word)
pub fn spawn_audio_task<L, P>(_on_level: L, _on_pcm: P) -> Result<()>
where
    L: Fn(f32) + Send + 'static,
    P: Fn(&[i16]) + Send + 'static,
{
    // TODO:
    //  1. I2S RX no BCLK=4, WS=8, DIN=6 (MEMS mic)
    //  2. I2S TX no BCLK=4, WS=8, DOUT=7 (DAC/amp)
    //  3. DMA circular 4×512 amostras
    //  4. Calcular RMS por bloco → on_level()
    //  5. Encaminhar PCM 16 kHz mono ao on_pcm()
    log::info!(
        "Áudio I2S task: stub inicializada ({} Hz / {} ch / {} bits)",
        SAMPLE_RATE, CHANNELS, BITS
    );
    Ok(())
}
