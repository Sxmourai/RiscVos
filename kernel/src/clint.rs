use core::time::Duration;
use crate::*;


pub enum CLINTRegs {
    MSIP,
    MTimeCMP,
    /// The MTIME register is a 64-bit read-write register that contains the number of cycles counted based on a fixed reference frequency.
    /// On MTIMER device reset, the MTIME register is cleared to zero.
    MTime,
}
impl CLINTRegs {
    pub fn addr<T>(self) -> *mut T {
        match self {
            Self::MSIP     => 0x0200_0000 as _,
            Self::MTimeCMP => 0x0200_4000 as _,
            Self::MTime    => 0x0200_BFF8 as _,
        }
    }
}
/// Maybe support next: (ACLINT)https://github.com/riscv/riscv-aclint/blob/main/riscv-aclint.adoc#machine-level-timer-device-mtimer
/// Good documentation: https://forums.sifive.com/t/documentation-for-clint/155/5
/// Or if you want is in the repo (SiFive e31 Manual)
pub struct CLINT;
impl CLINT {
    pub unsafe fn write<T>(&mut self, base: CLINTRegs, offset: usize, value: T) {
        unsafe {*(base.addr::<T>()).byte_add(offset) = value}
    }
    pub fn read<T: Copy>(&self, base: CLINTRegs, offset: usize) -> T {
        unsafe {*(base.addr::<T>()).byte_add(offset)}
    }
    pub fn current_time(&self) -> Duration {
        Duration::from_millis(self.current_cycles()/10_000)
    }
    /// Returns amount of cycles since last reset, 10_000_000/s on QEMU
    pub fn current_cycles(&self) -> u64 {
        self.read::<u64>(CLINTRegs::MTime, 0)
    }
    pub fn set_interrupt_enabled(&mut self, enabled: bool) {
        unsafe{self.write::<u32>(CLINTRegs::MSIP, 0, enabled as _)}
    }
    /// A timer interrupt is pending whenever
    /// mtime is greater than or equal to the value in the mtimecmp register. The timer interrupt is
    /// reflected in the mtip bit of the mip register described in Chapter 5
    pub fn set_mtimecmp(&mut self, value: u64) {
        unsafe {self.write(CLINTRegs::MTimeCMP, 0, value)}
    }

    /// Triggers an interrupt in `duration` **cycles**
    /// Try because if msip is set to 0 it won't work ! 
    pub fn try_trigger_in(&mut self, duration: Duration) {
        // 10k cycles/ms
        let cycles = duration.as_millis()*10_000;
        // Can't do nanoseconds tho...
        if cycles > u64::MAX as _ {todo!()}
        let cycles = cycles as u64;
        self.set_mtimecmp(self.current_cycles()+cycles)
    }
}


/// Initialises the machine timer,
/// Basically, we make it do an interrupt once, and then we re set it everytime in the interrupt handler
pub fn init() {
    handle_int()
}

pub fn handle_int() {
    CLINT.try_trigger_in(Duration::from_secs(1));
}