//! A simple console implementation using Noline
//!
//! This module implements console-like access to the
//! MCU program via USART1 on the DISCO board (over ST-LINK v2)
//! using Noline, which is a Rust library similar to readline
//! (for a prompt, command history, editing, etc.)
//!
//!

use embedded_io::{ErrorType, Read, Write};
use hal::gpio::{Alternate, PB7, PA9};
use hal::rcc::Clocks;
use hal::serial::{self, Rx, Tx, Serial};
use noline::error::NolineError;
use noline::builder::EditorBuilder;
use noline::sync_io::IO;
use stm32f7xx_hal as hal;
use stm32f7xx_hal::pac::USART1;
use stm32f7xx_hal::prelude::*;
use crate::app::serial_task;

pub struct SerialWrapper {
    rx: Rx<USART1>,
    tx: Tx<USART1>,
}

impl SerialWrapper {
    pub fn new(rx: serial::Rx<USART1>, tx: serial::Tx<USART1>) -> Self {
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
                        // Once a char is received, just return it
                        return Ok(1);
                    }
                    Err(_) => {}
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
                while let Err(_) = self.tx.write(*ch) {}
                sent_counter += 1;
            }
            Ok(sent_counter)
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

type RxPin = PB7<Alternate<7>>;
type TxPin = PA9<Alternate<7>>;

pub fn init_uart_serial(
    usart1: USART1,
    rx: RxPin,
    tx: TxPin,
    clocks: &Clocks,
) -> IO<SerialWrapper> {
    let serial = Serial::new(
        usart1,
        (tx, rx),
        &clocks,
	serial::Config::default(), // Default to 115_200 bauds
    );

    let (tx, rx) = serial.split();

    let wrapper = SerialWrapper::new(rx, tx);

    IO::new(wrapper)
}

pub async fn serial_task(cx: serial_task::Context<'_>) {
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
