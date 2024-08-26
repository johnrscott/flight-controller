//!
//!

use core::convert::Infallible;

use crate::app::serial_task;
use embedded_cli::cli::CliBuilder;
use embedded_cli::Command;
use embedded_io::{ErrorType, Read, Write};
use hal::gpio::{PA9, PB7};
use hal::rcc::Clocks;
use hal::serial::{self, Rx, Serial, Tx};
use stm32f7xx_hal as hal;
use stm32f7xx_hal::pac::USART1;
use stm32f7xx_hal::prelude::*;
use ufmt::uwrite;

#[derive(Command)]
enum Base<'a> {
    /// Say hello to World or someone else
    Hello {
        /// To whom to say hello (World by default)
        name: Option<&'a str>,
    },

    /// Stop CLI and exit
    Exit,
}

pub struct SerialTx {
    tx: Tx<USART1>,
}

impl SerialTx {
    pub fn new(tx: serial::Tx<USART1>) -> Self {
        Self { tx }
    }
}

struct SerialError {}

impl ErrorType for SerialTx {
    type Error = Infallible;
}

impl Write for SerialTx {
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

pub fn init_uart_serial(
    usart1: USART1,
    rx: PB7,
    tx: PA9,
    clocks: &Clocks,
) -> (Rx<USART1>, SerialTx) {
    let serial = Serial::new(
        usart1,
        (tx.into_alternate(), rx.into_alternate()),
        &clocks,
        serial::Config::default(), // Default to 115_200 bauds
    );

    let (tx, rx) = serial.split();

    (rx, SerialTx::new(tx))
}

pub async fn serial_task(cx: serial_task::Context<'_>) {
    defmt::info!("Starting serial task");
    let rx = cx.local.serial_rx;
    let tx = cx.local.serial_tx;

    // create static buffers for use in cli (so we're not using stack memory)
    // History buffer is 1 byte longer so max command fits in it (it requires
    // extra byte at end)
    // SAFETY: buffers are passed to cli and are used by cli only
    let (command_buffer, history_buffer) = unsafe {
        static mut COMMAND_BUFFER: [u8; 40] = [0; 40];
        static mut HISTORY_BUFFER: [u8; 41] = [0; 41];
        (COMMAND_BUFFER.as_mut(), HISTORY_BUFFER.as_mut())
    };

    let mut cli = CliBuilder::default()
        .writer(tx)
        .command_buffer(command_buffer)
        .history_buffer(history_buffer)
        .build()
        .unwrap();

    let _ = cli.write(|writer| {
        // storing big text in progmem
        // for small text it's usually better to use normal &str literals
        uwrite!(
            writer,
            "Cli is running.
Type \"help\" for a list of commands.
Use backspace and tab to remove chars and autocomplete.
Use up and down for history navigation.
Use left and right to move inside input."
        )
        .unwrap();
        Ok(())
    });

    loop {
        // Blocking loop waiting for character
        let byte = loop {
            if let Ok(ch) = rx.read() {
                break ch;
            }
        };

        let _ = cli.process_byte::<Base, _>(
            byte,
            &mut Base::processor(|cli, command| {
                match command {
                    Base::Hello { name } => {
                        // last write in command callback may or may not
                        // end with newline. so both uwrite!() and uwriteln!()
                        // will give identical results
                        uwrite!(cli.writer(), "Hello, {}", name.unwrap_or("World"))?;
                    }
                    Base::Exit => {
                        // We can write via normal function if formatting not needed
                        cli.writer().write_str("Cli can't shutdown now")?;
                    }
                }
                Ok(())
            }),
        );
    }
}
