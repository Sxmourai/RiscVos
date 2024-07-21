use crate::{print, println};

pub struct UART {
    base_port: *mut u8,
}
impl UART {
    /// # Safety
    /// Caller must ensure the `init` method is called next
    pub const unsafe fn new(port: u64) -> Self {
        
        Self {
            base_port: port as _,
        }
    }
    pub fn init(&mut self) {
        let port = self.base_port;
        unsafe { port.add(2).write_volatile(0b1) }; // enable FIFO 
        unsafe { port.add(1).write_volatile(0b1111) }; // Enable interrupts

        unsafe { port.add(3).write_volatile(1<<7) };
        // Also set bit to talk to the DLM (divisor latch most)
        // Setup signalling rate (BAUD)
        // For more info check: https://osblog.stephenmarz.com/ch2.html
        unsafe { port.add(0).write_volatile(0x50) }; // 592 & 0xFF = 0x50, easier cuz don't need casting
        unsafe { port.add(1).write_volatile((592>>8) as u8) };
        
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
        unsafe { self.base_port.write_volatile(chr) }
    }
    pub fn read_chr(&self) -> Option<u8> {
        unsafe {
            if self.base_port.add(5).read_volatile() & 1 != 0 { // Check DR bit
                Some(self.base_port.add(0).read_volatile())
            } else {None}
        }
    }
    pub fn handle_int(&self, ) {
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