use crate::uart_serial::init_uart_serial;
use stm32f7xx_hal::rcc::{self, HSEClock};
use stm32f7xx_hal::prelude::*;

use crate::app::Mono;
use crate::app::{init, Shared, Local};

const CLOCK_FREQ: u32 = 216_000_000;

pub fn init(cx: init::Context) -> (Shared, Local) {
    defmt::info!("Starting RTIC init task");

    Mono::start(cx.core.SYST, CLOCK_FREQ);

    // Device specific peripherals
    let device = cx.device;

    // The DISCO board has a 25 MHz oscillator connected to
    // the HSE input. Configure the MCU to use this external
    // oscillator, and then set a frequency between 12.5 MHz
    // and 216 MHz (the program will panic if out of range).
    let hse_cfg = HSEClock::new(25_000_000.Hz(), rcc::HSEClockMode::Bypass);
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.hse(hse_cfg).sysclk(CLOCK_FREQ.Hz()).freeze();

    let gpioa = device.GPIOA.split();
    let gpiob = device.GPIOB.split();

    let rx = gpiob.pb7.into_alternate();
    let tx = gpioa.pa9.into_alternate();
    let usart1 = device.USART1;
    let io = init_uart_serial(usart1, rx, tx, &clocks);
    
    crate::app::hello_loop::spawn().ok();
    crate::app::serial_task::spawn().ok();

    defmt::info!("Ending init task");
    
    (Shared {}, Local { io })
}
