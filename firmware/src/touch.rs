//! Task do touch FT6336G — I2C0 + INT + tradução de gestos.
//! Publica `TouchEvent` num canal MPSC consumido pelo main loop Slint.

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub struct Point { pub x: i16, pub y: i16 }

#[derive(Debug, Clone, Copy)]
pub enum TouchEvent {
    Down(Point),
    Move(Point),
    Up(Point),
    SwipeX { delta: i16 }, // navegação horizontal (launcher)
    SwipeY { delta: i16 }, // painel superior (control center) / dismiss
}

pub const FT6336G_ADDR: u8 = 0x38;

/// Loop-de-tarefa (FreeRTOS): configura I2C0 nas linhas SDA/SCL,
/// habilita a interrupção em INT, faz polling do controlador
/// (registros 0x02..0x0F) e emite eventos.
pub fn spawn_touch_task<F>(_on_event: F) -> Result<()>
where
    F: Fn(TouchEvent) + Send + 'static,
{
    // TODO(bring-up):
    //   1. I2cDriver::new(i2c0, SDA=16, SCL=15, 400 kHz)
    //   2. PinDriver::input(TOUCH_INT).subscribe(...)
    //   3. read regs [ 0x02: touch_count, 0x03..0x06: x/y do dedo 1, ... ]
    //   4. buffer últimos N pontos → derivar SwipeX/SwipeY.
    log::info!("Touch FT6336G task: stub inicializada (I2C @400kHz)");
    Ok(())
}
