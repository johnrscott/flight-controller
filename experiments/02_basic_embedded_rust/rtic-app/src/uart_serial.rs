use embedded_io::{ErrorType, Read, Write};
use hal::serial::{self, Rx, Tx};
use noline::error::NolineError;
use stm32f7xx_hal as hal;
use stm32f7xx_hal::pac::USART1;
use stm32f7xx_hal::prelude::*;

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
