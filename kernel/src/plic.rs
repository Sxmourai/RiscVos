use crate::*;

pub enum PLICRegs {
    /// Sets the priority of a particular interrupt source
    Priority(usize),
    /// Contains a list of interrupts that have been triggered (are pending)
    /// Read-Only
    Pending(usize),
    /// Enable/disable certain interrupt sources
    Enable(usize),
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
            Self::Priority(i)  =>  {let addr = (0x0c00_0000+4*i); assert!(i<=127 && i>0); addr as _}, 
            Self::Pending(i)   =>  {let addr = (0x0c00_1000+4*i); assert!(i<=3); addr as _}, 
            Self::Enable(i)    =>  {let addr = (0x0c00_2000+4*i); assert!(i<=3); addr as _}, 
            Self::Threshold =>  0x0c20_0000 as _, 
            Self::Claim     =>  0x0c20_0004 as _, 
            Self::Complete  =>  0x0c20_0004 as _, 
        }
    }
}

pub struct PLIC;
impl PLIC {
    pub fn write(&mut self, base: PLICRegs, value: u32) {
        unsafe {*(base.addr()) = value}
    }
    pub fn read(&self, base: PLICRegs) -> u32 {
        unsafe {*(base.addr())}
    }
    pub fn enable_interrupt(&mut self, int: usize) {
        let interrupts: u32 = self.read(PLICRegs::Enable(0));
        self.write(PLICRegs::Enable(0), interrupts | (1<<int));
    }
    /// Priority should be <= 7
    pub fn set_priority(&mut self, int: usize, priority: u8) {
        assert_eq!(priority, priority&0b111); // Debug for now, we will want to do a silent cast later ?
        self.write(PLICRegs::Priority(int), priority as u32);
    }
    /// If an interrupt's priority < threshold, it is ignored 
    pub fn set_threshold(&mut self, threshold: u8) {
        assert_eq!(threshold, threshold&0b111); // Debug for now, we will want to do a silent cast later ?
        self.write(PLICRegs::Threshold, threshold as u32);
    }
    /// Could return plain value, which would be 0 instead of None
    /// But this option forces us to unwrap the value and process it
    /// 
    /// Doesn't need mut self, but we are "simulating" the real behavior of the PLIC, 
    /// The PLIC stops listening to other ints when we claim, so we change the state
    pub fn claim_int(&mut self) -> Option<u32> {
        let int = self.read(PLICRegs::Claim);
        if int == 0 {return None}
        Some(int)
    }
    /// # Safety
    /// Caller must ensure this interrupt has been trigger
    pub unsafe fn eoi(&mut self, int: u32) {
        self.write(PLICRegs::Complete, int);
    }
}

pub fn init() {
    info!("Initialising PLIC...");
    PLIC.enable_interrupt(10); // UART0 interrupt (see qemu source code)
    for i in 1..=8 { // Virt IO
        PLIC.enable_interrupt(i);
        PLIC.set_priority(i, 1);
    }
    PLIC.set_priority(10, 2);
    PLIC.set_threshold(0);
}