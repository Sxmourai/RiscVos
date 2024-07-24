use bit_field::BitField;

use crate::{print, println};

pub enum UARTInts {
    /// ERBFI
    ReceivedDataAvalaible = 1<<0,
    /// ETBEI
    TransmitterHoldingRegisterEmpty = 1<<1,
    /// ELSI
    ReceiverLineStatus = 1<<2,
    /// EDSSI
    MODEMStatus = 1<<3,
    /// Can't enable this interrupt, so no value, but can be raised
    CharacterTimeoutIndication,
}
impl UARTInts {
    pub fn all() -> u8 {0b1111}
    pub fn parse(s: u8) -> Option<Self> {
        Some(match s {
            0x00=>Self::MODEMStatus,
            0x02=>Self::TransmitterHoldingRegisterEmpty,
            0x04=>Self::ReceivedDataAvalaible,
            0x06=>Self::ReceiverLineStatus,
            0x0C=>Self::CharacterTimeoutIndication,
            _ => return None
        })
    }
}


pub struct UART {
    base_port: u64,
}
impl UART {
    /// # Safety
    /// Caller must ensure the `init` method is called next
    /// Caller must ensure the address is right and not pointing to random stuff in memory
    /// which could cause page faults & co
    pub const unsafe fn new(port: u64) -> Self {
        
        Self {
            base_port: port as _,
        }
    }
    // From QEMU source code, clock frequency is 3686400 https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c#L946
    // UART source: https://github.com/qemu/qemu/blob/master/hw/char/serial.c
    pub fn init(&mut self) {
        let port = self.base_port as *mut u8;
        unsafe { port.add(2).write_volatile(0b1) }; // enable FIFO 
        unsafe { port.add(1).write_volatile(UARTInts::ReceivedDataAvalaible as _) };

        // Set divisor latch access bit to 1, makes registers at addr port+0 and port+1 be DLL and DLM.
        unsafe { port.add(3).write_volatile(1<<7) };
        // Setup signalling rate (BAUD)
        // For more info check: https://osblog.stephenmarz.com/ch2.html
        unsafe { port.add(0).write_volatile(0x50) }; // 592 & 0xFF = 0x50, easier cuz don't need casting
        unsafe { port.add(1).write_volatile((592>>8) as u8) };
        
        // Stop talking to divisor latch & set word length to 8 bits.
        unsafe { port.add(3).write_volatile(0b1 | 0b10) }; // set word length to 8bits, instead of default 5-bits (even tho 8 by default on QEMU)
    }
    // We take "mut" self because we write so it's kinda mutating something, even though we don't need it
    pub fn write(&mut self, str: impl Iterator<Item = u8>) {
        for char in str {
            self.write_chr(char);
        }
    }
    // #[inline(always)]
    pub fn write_chr(&mut self, chr: u8) {
        unsafe { (self.base_port as *mut u8).write_volatile(chr) }
    }
    pub fn read_chr(&self) -> Option<u8> {
        unsafe {
            if (self.base_port as *const u8).add(5).read_volatile() & 1 != 0 { // Check DR bit
                Some((self.base_port as *const u8).add(0).read_volatile())
            } else {None}
        }
    }
    pub fn get_int(&self) -> Option<UARTInts> {
        let int_ident = unsafe{(self.base_port as *const u8).add(2).read_volatile()};
        if int_ident.get_bit(0) {return None;} // Should be 0 (or false) if interrupt is pending
        UARTInts::parse((int_ident)&0b111)
    }
    pub fn handle_int(&self, ) {
        if let Some(int) = self.get_int() {
            match int {
                UARTInts::ReceivedDataAvalaible => self.received_data(),
                UARTInts::TransmitterHoldingRegisterEmpty => todo!(),
                UARTInts::ReceiverLineStatus => todo!(),
                UARTInts::MODEMStatus => todo!(),
                UARTInts::CharacterTimeoutIndication => todo!(),
            }
        }
    }
    pub fn received_data(&self) {
        let input_char = self.read_chr();
        if let Some(input_char) = input_char {
            match input_char {
                127 => { // Backspace
                    print!("{}{}{}", 8 as char, ' ', 8 as char);
                },
                10 | 13 => {
                    println!();
                },
                3 => {// From what i've seen it's CTRL+C
                    crate::tests::close_qemu()
                },
                _ => {
                    print!("{}", input_char as char);
                }
            }
        }
    }
}
impl core::fmt::Write for UART {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.bytes());
        Ok(())
    }
}