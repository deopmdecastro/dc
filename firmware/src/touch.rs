//! Driver do touch capacitivo FT6336G (I2C @ 0x38).
//! Le o ponto ativo por polling e envia eventos para o loop Slint.

use crate::pinout::{pins, DISPLAY_H, DISPLAY_W};
use anyhow::Result;
use esp_idf_hal::{
    delay::{FreeRtos, BLOCK},
    gpio::{AnyIOPin, Input, PinDriver, Pull},
    i2c::{I2cConfig, I2cDriver, I2C0},
    units::*,
};
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone, Copy)]
pub enum TouchEvent {
    Down(Point),
    Move(Point),
    Up(Point),
    SwipeX { delta: i16 },
    SwipeY { delta: i16 },
}

pub const FT6336G_ADDR: u8 = 0x38;

const TOUCH_TASK_STACK_SIZE: usize = 6 * 1024;
const REG_TOUCH_COUNT: u8 = 0x02;
const POLL_MS: u32 = 20;

/// Spawna a task de touch. O loop principal consome o `Receiver` retornado e
/// injeta os eventos no Slint como eventos de ponteiro.
pub fn spawn_touch_task(i2c0: I2C0<'static>) -> Result<Receiver<TouchEvent>> {
    let (tx, rx) = mpsc::channel();

    std::thread::Builder::new()
        .name("dc-touch".into())
        .stack_size(TOUCH_TASK_STACK_SIZE)
        .spawn(move || {
            if let Err(e) = run_touch(i2c0, tx) {
                log::error!("Touch FT6336G task terminou: {e:?}");
            }
        })?;

    Ok(rx)
}

fn run_touch(i2c0: I2C0<'static>, tx: Sender<TouchEvent>) -> Result<()> {
    let mut rst = PinDriver::output(unsafe { AnyIOPin::steal(pins::TOUCH_RST as _) })?;
    rst.set_low()?;
    FreeRtos::delay_ms(10);
    rst.set_high()?;
    FreeRtos::delay_ms(300);

    let _int: PinDriver<'_, Input> =
        PinDriver::input(unsafe { AnyIOPin::steal(pins::TOUCH_INT as _) }, Pull::Up)?;

    let sda = unsafe { AnyIOPin::steal(pins::TOUCH_SDA as _) };
    let scl = unsafe { AnyIOPin::steal(pins::TOUCH_SCL as _) };
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let mut i2c = I2cDriver::new(i2c0, sda, scl, &config)?;

    let mut chip_id = [0_u8; 1];
    match i2c.write_read(FT6336G_ADDR, &[0xA8], &mut chip_id, BLOCK) {
        Ok(()) => log::info!("Touch FT6336G: I2C OK, chip/vendor id=0x{:02X}", chip_id[0]),
        Err(e) => log::warn!("Touch FT6336G: leitura de chip id falhou: {e:?}"),
    }

    log::info!(
        "Touch FT6336G task: polling {} ms, SDA={}, SCL={}, RST={}, INT={}",
        POLL_MS,
        pins::TOUCH_SDA,
        pins::TOUCH_SCL,
        pins::TOUCH_RST,
        pins::TOUCH_INT
    );

    let mut pressed = false;
    let mut last_point = Point { x: 0, y: 0 };
    let mut read_errors = 0_u32;

    loop {
        match read_touch_point(&mut i2c) {
            Ok(Some((raw_x, raw_y, point))) => {
                let event = if pressed {
                    if point == last_point {
                        None
                    } else {
                        Some(TouchEvent::Move(point))
                    }
                } else {
                    Some(TouchEvent::Down(point))
                };

                if let Some(event) = event {
                    if tx.send(event).is_err() {
                        break;
                    }

                    if !pressed {
                        log::info!(
                            "Touch: down raw=({}, {}) ui=({}, {})",
                            raw_x,
                            raw_y,
                            point.x,
                            point.y
                        );
                    }
                }

                pressed = true;
                last_point = point;
                read_errors = 0;
            }
            Ok(None) => {
                if pressed {
                    let _ = tx.send(TouchEvent::Up(last_point));
                    log::info!("Touch: up ui=({}, {})", last_point.x, last_point.y);
                }

                pressed = false;
                read_errors = 0;
            }
            Err(e) => {
                read_errors = read_errors.wrapping_add(1);
                if read_errors == 1 || read_errors % 100 == 0 {
                    log::warn!("Touch FT6336G: falha I2C #{read_errors}: {e:?}");
                }
            }
        }

        FreeRtos::delay_ms(POLL_MS);
    }

    Ok(())
}

fn read_touch_point(i2c: &mut I2cDriver<'_>) -> Result<Option<(u16, u16, Point)>> {
    let mut buf = [0_u8; 5];
    i2c.write_read(FT6336G_ADDR, &[REG_TOUCH_COUNT], &mut buf, BLOCK)?;

    let touches = buf[0] & 0x0F;
    if touches == 0 || touches > 2 {
        return Ok(None);
    }

    let raw_x = (((buf[1] & 0x0F) as u16) << 8) | buf[2] as u16;
    let raw_y = (((buf[3] & 0x0F) as u16) << 8) | buf[4] as u16;

    Ok(Some((raw_x, raw_y, map_touch_to_landscape(raw_x, raw_y))))
}

fn map_touch_to_landscape(raw_x: u16, raw_y: u16) -> Point {
    let max_x = DISPLAY_W as i16 - 1;
    let max_y = DISPLAY_H as i16 - 1;
    let x = (raw_y as i16).clamp(0, max_x);
    let y = (max_y - raw_x as i16).clamp(0, max_y);

    Point { x, y }
}
