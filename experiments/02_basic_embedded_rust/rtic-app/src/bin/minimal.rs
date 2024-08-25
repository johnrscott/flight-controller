#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use test_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [TIM2, TIM3])]
mod app {

    const CLOCK_FREQ: u32 = 216_000_000;
    
    use hal::pac::USART1;
    use hal::rcc::{self, HSEClock};
    use hal::serial::{self, Serial, Rx, Tx};
    use noline::builder::EditorBuilder;
    use noline::error::NolineError;
    use noline::sync_io::IO;
    use stm32f7xx_hal as hal;
    use stm32f7xx_hal::prelude::*;
    use embedded_io::{Read, Write, ErrorType};
    
    use rtic_monotonics::systick::prelude::*;

    systick_monotonic!(Mono, 100);
    
    #[shared]
    struct Shared {
    }

    #[local]
    struct Local {
	io: IO<SerialWrapper>
    }

    struct SerialWrapper {
	rx: Rx<USART1>,	
	tx: Tx<USART1>,
    }

    impl SerialWrapper {

	fn new(rx: serial::Rx<USART1>, tx: serial::Tx<USART1>) -> Self {
	    Self { rx, tx }
	}
    }

    impl ErrorType for SerialWrapper {
	type Error = NolineError;
    }
    
    impl Read for SerialWrapper {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
	    // Really basic implementation, just one char at a time
	    if buf.len() == 0 {
		Ok(0)
	    } else {

		// This function blocks, so just wait for char
		loop {
		    match self.rx.read() {
			Ok(ch) => {			    
			    buf[0] = ch;
			    defmt::info!("Read: {}", ch);
			    // Once a char is received, just return it
			    return Ok(1);
			},
			Err(_) => {},
		    }
		}
	    }
	}
    }

    impl Write for SerialWrapper {
	fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
	    if buf.len() == 0 {
		Ok(0)
	    } else {
		let mut sent_counter: usize = 0;
		for ch in buf {
		    // Loop calling write until it succeeds. The HAL
		    // serial write call does not block if a character
		    // is currently being transmitted; it returns without
		    // sending anything. Keep retrying until ch is sent.
		    //while let Err(_) = self.tx.write(*ch) {};
		    
		    defmt::info!("write: {}", *ch);
		    sent_counter += 1;
		}
		Ok(sent_counter)
	    }
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
	    Ok(())
	}
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

	let gpioa = device.GPIOA.split();
	let gpiob = device.GPIOB.split();
	
	let tx = gpioa.pa9.into_alternate();
	let rx = gpiob.pb7.into_alternate();

	let serial = Serial::new(
            device.USART1,
            (tx, rx),
            &clocks,
            serial::Config {
		// Default to 115_200 bauds
		..Default::default()
            },
	);

	let (tx, rx) = serial.split();

	let mut wrapper = SerialWrapper::new(rx, tx);
	// let mut buf: [u8; 16] = ['t' as u8; 16];
	// wrapper.write(&mut buf).ok();
	//let mut buf: [u8; 16] = [0; 16];  
	// loop {
	//     wrapper.read(&mut buf);
	//     defmt::info!("{}", buf);
	// }
	
	let io = IO::new(wrapper);
	
        //hello_loop::spawn().ok();
	serial_task::spawn().ok();

	defmt::info!("Ending init task");
	
        (Shared {}, Local {io})
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }
    
    #[task(priority = 1)]
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
	let mut editor = EditorBuilder::new_static::<128>()
            .with_static_history::<128>()
            .build_sync(&mut io)
            .unwrap();
	
	while let Ok(line) = editor.readline("> ", &mut io) {
            defmt::info!("Read: '{}'", line);
	}
    }    
}
