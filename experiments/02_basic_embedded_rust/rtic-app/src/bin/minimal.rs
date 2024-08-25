#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use app_lib; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [TIM2, TIM3])]
mod app {

    const CLOCK_FREQ: u32 = 216_000_000;

    use app_lib::uart_serial::SerialWrapper;
    use hal::gpio::{Pin, Alternate};
    use hal::pac::{Peripherals, USART1, GPIOA, GPIOB};
    use hal::rcc::{self, Clocks, HSEClock};
    use hal::serial::{self, Serial, PinRx, PinTx};
    use noline::builder::EditorBuilder;
    use noline::sync_io::IO;
    use stm32f7xx_hal as hal;
    use stm32f7xx_hal::prelude::*;

    use rtic_monotonics::systick::prelude::*;

    systick_monotonic!(Mono, 100);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        io: IO<SerialWrapper>,
    }

    fn init_uart_serial<MODE>(usart1: USART1, rx: Pin<'B', 7, MODE>, tx: Pin<'A', 9, MODE>, clocks: &Clocks) -> IO<SerialWrapper> {
	
        let serial = Serial::new(
            usart1,
            (tx, rx),
            &clocks,
            serial::Config {
                // Default to 115_200 bauds
                ..Default::default()
            },
        );

        let (tx, rx) = serial.split();

        let wrapper = SerialWrapper::new(rx, tx);
        IO::new(wrapper)
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
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

        hello_loop::spawn().ok();
        serial_task::spawn().ok();

        defmt::info!("Ending init task");

        let gpioa = device.GPIOA.split();
        let gpiob = device.GPIOB.split();
        let rx = gpiob.pb7.into_push_pull_output();
        let tx = gpioa.pa9.into_push_pull_output();
	let usart1 = device.USART1;
	let io = init_uart_serial(usart1, rx, tx, &clocks);
	
        (Shared {}, Local { io })
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(priority = 2)]
    async fn hello_loop(_cx: hello_loop::Context) {
        loop {
            Mono::delay(1.secs()).await;
            defmt::info!("Hello every 1s!");
        }
    }

    #[task(priority = 1, local=[io])]
    async fn serial_task(cx: serial_task::Context) {
        defmt::info!("Starting serial task");
        let mut io = cx.local.io;

        let mut fail_count = 0;
        let mut editor = loop {
            match EditorBuilder::new_static::<256>()
                .with_static_history::<256>()
                .build_sync(&mut io)
            {
                Ok(editor) => {
                    defmt::info!("Successfully configured serial prompt");
                    break editor;
                }
                Err(_) => {
                    defmt::warn!(
                        "Failed to initialise serial prompt ({}). Re-trying",
                        fail_count
                    );
                    fail_count += 1;
                }
            };
        };

        while let Ok(line) = editor.readline("MCU $ ", &mut io) {
            defmt::info!("Received command: '{}'", line);
        }
    }
}
