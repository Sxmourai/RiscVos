use crate::*;

pub enum PLICRegs {
    /// Sets the priority of a particular interrupt source
    Priority,
    /// Contains a list of interrupts that have been triggered (are pending)
    Pending,
    /// Enable/disable certain interrupt sources
    Enable,
    /// Sets the threshold that interrupts must meet before being able to trigger.
    Threshold,
    /// Returns the next interrupt in priority order.
    Claim,
    /// Completes handling of a particular interrupt.
    Complete,
}
impl PLICRegs {
    pub fn addr(self) -> *mut u32 {
        match self {
            Self::Priority  =>  0x0c00_0000 as _, 
            Self::Pending   =>  0x0c00_1000 as _, 
            Self::Enable    =>  0x0c00_2000 as _, 
            Self::Threshold =>  0x0c20_0000 as _, 
            Self::Claim     =>  0x0c20_0004 as _, 
            Self::Complete  =>  0x0c20_0004 as _, 
        }
    }
}

pub struct PLIC;
impl PLIC {
    pub unsafe fn write(&mut self, base: PLICRegs, offset: usize, value: u32) {
        unsafe {*(base.addr()).byte_add(offset) = value}
    }
    pub fn read(&self, base: PLICRegs, offset: usize) -> u32 {
        unsafe {*(base.addr()).byte_add(offset)}
    }
    pub fn enable_interrupt(&mut self, int: usize) {
        let interrupts: u32 = unsafe{self.read(PLICRegs::Enable, 0, )};
        unsafe{self.write(PLICRegs::Enable, 0, interrupts | (1<<int))}
    }
    /// Priority should be <= 7
    pub fn set_priority(&mut self, int: usize, priority: u8) {
        assert_eq!(priority, priority&0b111); // Debug for now, we will want to do a silent cast later ?
        let priorities: u32 = unsafe{self.read(PLICRegs::Priority, 0, )};
        unsafe{self.write(PLICRegs::Priority, 4*int, priority as u32)}
    }
    /// If an interrupt's priority < threshold, it is ignored 
    pub fn set_threshold(&mut self, threshold: u8) {
        assert_eq!(threshold, threshold&0b111); // Debug for now, we will want to do a silent cast later ?
        let priorities: u32 = unsafe{self.read(PLICRegs::Threshold, 0, )};
        unsafe{self.write(PLICRegs::Threshold, 0, threshold as u32)}
    }
    /// Could return plain value, which would be 0 instead of None
    /// But this option forces us to unwrap the value and process it
    /// Doesn't need mut self, but we are "simulating" the real behavior of the PLIC, 
    /// The PLIC stops listening to other ints when we claim, so we change the state
    pub fn claim_int(&mut self) -> Option<u32> {
        let int = unsafe {self.read(PLICRegs::Claim, 0)};
        if int == 0 {return None}
        Some(int)
    }
    pub fn eoi(&mut self, int: u32) {
        unsafe {self.write(PLICRegs::Complete, 0, int)};
    }
}

pub fn init() {
    println!("Initialising PLIC...");
    PLIC.enable_interrupt(10); // UART0 interrupt (see qemu source code)
    PLIC.set_priority(10, 2);
    PLIC.set_threshold(0);
}