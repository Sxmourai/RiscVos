use std::fmt::Display;

use super::uguest;

fn todo_write(id: CsrID, csr: uguest) -> uguest {
    println!("WARN: writing to unsupported csr {id}");
    csr
}
pub type CsrWriteHandler = fn(id: CsrID, uguest) -> uguest;
pub const CSRS: [CsrWriteHandler; 4096] = [todo_write; 4096];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[allow(non_camel_case_types)]
// ChatGPT is pretty good for this =)
pub enum SupportedCsrID {
    // Machine Information Registers
    mvendorid = 0xF11, // "MRO", "Vendor ID."),
    marchid = 0xF12, // "MRO", "Architecture ID."),
    mimpid = 0xF13, // "MRO", "Implementation ID."),
    mhartid = 0xF14, // "MRO", "Hardware thread ID."),
    mconfigptr = 0xF15, // "MRO", "Pointer to configuration data structure."),

    // Machine Trap Setup
    mstatus = 0x300, // "MRW", "Machine status register."),
    misa = 0x301, // "MRW", "ISA and extensions"),
    medeleg = 0x302, // "MRW", "Machine exception delegation register."),
    mideleg = 0x303, // "MRW", "Machine interrupt delegation register."),
    mie = 0x304, // "MRW", "Machine interrupt-enable register."),
    mtvec = 0x305, // "MRW", "Machine trap-handler base address."),
    mcounteren = 0x306, // "MRW", "Machine counter enable."),
    mstatush = 0x310, // "MRW", "Additional machine status register, RV32 only."),
    medelegh = 0x312, // "MRW", "Upper 32 bits of medeleg, RV32 only."),

    // Machine Trap Handling
    mscratch = 0x340, // "MRW", "Scratch register for machine trap handlers."),
    mepc = 0x341, // "MRW", "Machine exception program counter."),
    mcause = 0x342, // "MRW", "Machine trap cause."),
    mtval = 0x343, // "MRW", "Machine bad address or instruction."),
    mip = 0x344, // "MRW", "Machine interrupt pending."),
    mtinst = 0x34A, // "MRW", "Machine trap instruction (transformed)."),
    mtval2 = 0x34B, // "MRW", "Machine bad guest physical address."),

    // Machine Configuration
    menvcfg = 0x30A, // "MRW", "Machine environment configuration register."),
    menvcfgh = 0x31A, // "MRW", "Upper 32 bits of menvcfg, RV32 only."),
    mseccfg = 0x747, // "MRW", "Machine security configuration register."),
    mseccfgh = 0x757, // "MRW", "Upper 32 bits of mseccfg, RV32 only."),

    // Machine Memory Protection
    pmpcfg0 = 0x3A0, // "MRW", "Physical memory protection configuration."),
    pmpcfg1 = 0x3A1, // "MRW", "Physical memory protection configuration, RV32 only."),
    pmpcfg2 = 0x3A2, // "MRW", "Physical memory protection configuration."),
    pmpcfg3 = 0x3A3, // "MRW", "Physical memory protection configuration, RV32 only."),
    pmpcfg14 = 0x3AE, // "MRW", "Physical memory protection configuration."),
    pmpcfg15 = 0x3AF, // "MRW", "Physical memory protection configuration, RV32 only."),
    pmpaddr0 = 0x3B0, // "MRW", "Physical memory protection address register."),
    pmpaddr1 = 0x3B1, // "MRW", "Physical memory protection address register."),
    pmpaddr63 = 0x3EF, // "MRW", "Physical memory protection address register."),

    // Machine State Enable Registers
    mstateen0 = 0x30C, // "MRW", "Machine State Enable 0 Register."),
    mstateen1 = 0x30D, // "MRW", "Machine State Enable 1 Register."),
    mstateen2 = 0x30E, // "MRW", "Machine State Enable 2 Register."),
    mstateen3 = 0x30F, // "MRW", "Machine State Enable 3 Register."),
    mstateen0h = 0x31C, // "MRW", "Upper 32 bits of Machine State Enable 0 Register, RV32 only."),
    mstateen1h = 0x31D, // "MRW", "Upper 32 bits of Machine State Enable 1 Register, RV32 only."),
    mstateen2h = 0x31E, // "MRW", "Upper 32 bits of Machine State Enable 2 Register, RV32 only."),
    mstateen3h = 0x31F, // "MRW", "Upper 32 bits of Machine State Enable 3 Register, RV32 only."),
    fflags = 0x001, // "URW", "Floating-Point Accrued Exceptions."),
    frm = 0x002, // "URW", "Floating-Point Dynamic Rounding Mode."),
    fcsr = 0x003, // "URW", "Floating-Point Control and Status Register (frm +fflags)."),
            
    // Unprivileged Counter/Timers
    cycle = 0xC00, // "URO", "Cycle counter for RDCYCLE instruction."),
    time = 0xC01, // "URO", "Timer for RDTIME instruction."),
    instret = 0xC02, // "URO", "Instructions-retired counter for RDINSTRET instruction."),
    hpmcounter3 = 0xC03, // "URO", "Performance-monitoring counter."),
    hpmcounter4 = 0xC04, // "URO", "Performance-monitoring counter."),
    // ... and so on for other unprivileged CSRs
            
    // Supervisor-level CSRs
    sstatus = 0x100, // "SRW", "Supervisor status register."),
    sie = 0x104, // "SRW", "Supervisor interrupt-enable register."),
    stvec = 0x105, // "SRW", "Supervisor trap handler base address."),
    scounteren = 0x106, // "SRW", "Supervisor counter enable."),
    senvcfg = 0x10A, // "SRW", "Supervisor environment configuration register."),
    scountinhibit = 0x120, // "SRW", "Supervisor counter-inhibit register."),
    sscratch = 0x140, // "SRW", "Scratch register for supervisor trap handlers."),
    sepc = 0x141, // "SRW", "Supervisor exception program counter."),
    scause = 0x142, // "SRW", "Supervisor trap cause."),
    stval = 0x143, // "SRW", "Supervisor bad address or instruction."),
    sip = 0x144, // "SRW", "Supervisor interrupt pending."),
    scountovf = 0xDA0, // "SRO", "Supervisor count overflow."),
    satp = 0x180, // "SRW", "Supervisor address translation and protection."),
    scontext = 0x5A8, // "SRW", "Supervisor-mode context register."),
    sstateen0 = 0x10C, // "SRW", "Supervisor State Enable 0 Register."),
    sstateen1 = 0x10D, // "SRW", "Supervisor State Enable 1 Register."),
    sstateen2 = 0x10E, // "SRW", "Supervisor State Enable 2 Register."),
    sstateen3 = 0x10F, // "SRW", "Supervisor State Enable 3 Register."),
            
    // Hypervisor-level CSRs
    hstatus = 0x600, // "HRW", "Hypervisor status register."),
    hedeleg = 0x602, // "HRW", "Hypervisor exception delegation register."),
    hideleg = 0x603, // "HRW", "Hypervisor interrupt delegation register."),
    hie = 0x604, // "HRW", "Hypervisor interrupt-enable register."),
    hcounteren = 0x606, // "HRW", "Hypervisor counter enable."),
    hgeie = 0x607, // "HRW", "Hypervisor guest external interrupt-enable register."),
    hedelegh = 0x612, // "HRW", "Upper 32 bits of hedeleg, RV32 only."),
    htval = 0x643, // "HRW", "Hypervisor bad guest physical address."),
    hip = 0x644, // "HRW", "Hypervisor interrupt pending."),
    hvip = 0x645, // "HRW", "Hypervisor virtual interrupt pending."),
    htinst = 0x64A, // "HRW", "Hypervisor trap instruction (transformed)."),
    hgeip = 0xE12, // "HRO", "Hypervisor guest external interrupt pending."),
    henvcfg = 0x60A, // "HRW", "Hypervisor environment configuration register."),
    henvcfgh = 0x61A, // "HRM", "Upper 32 bits of henvcfg, RV32 only."),
    hgatp = 0x680, // "HRW", "Hypervisor guest address translation and protection."),
    hcontext = 0x6A8, // "HRW", "Hypervisor-mode context register."),
    htimedelta = 0x605, // "HRW", "Delta for VS/VU-mode timer."),
    htimedeltah = 0x615, // "HRW", "Upper 32 bits of htimedelta, RV32 only."),
    hstateen0 = 0x60C, // "HRW", "Hypervisor State Enable 0 Register."),
    hstateen1 = 0x60D, // "HRW", "Hypervisor State Enable 1 Register."),
    hstateen2 = 0x60E, // "HRW", "Hypervisor State Enable 2 Register."),
    hstateen3 = 0x60F, // "HRW", "Hypervisor State Enable 3 Register."),
    hstateen0h = 0x61C, // "HRW", "Upper 32 bits of Hypervisor State Enable 0 Register, RV32 only."),
    hstateen1h = 0x61D, // "HRW", "Upper 32 bits of Hypervisor State Enable 1 Register, RV32 only."),
    hstateen2h = 0x61E, // "HRW", "Upper 32 bits of Hypervisor State Enable 2 Register, RV32 only."),
    hstateen3h = 0x61F, // "HRW", "Upper 32 bits of Hypervisor State Enable 3 Register, RV32 only."),
            
    // Virtual Supervisor Registers
    vsstatus = 0x200, // "HRW", "Virtual supervisor status register."),
    vsie = 0x204, // "HRW", "Virtual supervisor interrupt-enable register."),
    vstvec = 0x205, // "HRW", "Virtual supervisor trap handler base address."),
    vsscratch = 0x240, // "HRW", "Virtual supervisor scratch register."),
    vsepc = 0x241, // "HRW", "Virtual supervisor exception program counter."),
    vscause = 0x242, // "HRW", "Virtual supervisor trap cause."),
    vstval = 0x243, // "HRW", "Virtual supervisor bad address or instruction."),
    vsip = 0x244, // "HRW", "Virtual supervisor interrupt pending."),
    vsatp = 0x280, // "HRW", "Virtual supervisor address translation and protection."),

    // Unsupported(u16),
}
const CSR_TABLE: [Option<SupportedCsrID>; 0x1000] = {
    let mut table = [None; 0x1000];
    
    table[0xF11] = Some(SupportedCsrID::mvendorid);
    table[0xF12] = Some(SupportedCsrID::marchid);
    table[0xF13] = Some(SupportedCsrID::mimpid);
    table[0xF14] = Some(SupportedCsrID::mhartid);
    table[0xF15] = Some(SupportedCsrID::mconfigptr);

    table[0x300] = Some(SupportedCsrID::mstatus);
    table[0x301] = Some(SupportedCsrID::misa);
    table[0x302] = Some(SupportedCsrID::medeleg);
    table[0x303] = Some(SupportedCsrID::mideleg);
    table[0x304] = Some(SupportedCsrID::mie);
    table[0x305] = Some(SupportedCsrID::mtvec);
    table[0x306] = Some(SupportedCsrID::mcounteren);
    table[0x310] = Some(SupportedCsrID::mstatush);
    table[0x312] = Some(SupportedCsrID::medelegh);

    table[0x340] = Some(SupportedCsrID::mscratch);
    table[0x341] = Some(SupportedCsrID::mepc);
    table[0x342] = Some(SupportedCsrID::mcause);
    table[0x343] = Some(SupportedCsrID::mtval);
    table[0x344] = Some(SupportedCsrID::mip);
    table[0x34A] = Some(SupportedCsrID::mtinst);
    table[0x34B] = Some(SupportedCsrID::mtval2);

    table[0x30A] = Some(SupportedCsrID::menvcfg);
    table[0x31A] = Some(SupportedCsrID::menvcfgh);
    table[0x747] = Some(SupportedCsrID::mseccfg);
    table[0x757] = Some(SupportedCsrID::mseccfgh);

    table[0x3A0] = Some(SupportedCsrID::pmpcfg0);
    table[0x3A1] = Some(SupportedCsrID::pmpcfg1);
    table[0x3A2] = Some(SupportedCsrID::pmpcfg2);
    table[0x3A3] = Some(SupportedCsrID::pmpcfg3);
    table[0x3AE] = Some(SupportedCsrID::pmpcfg14);
    table[0x3AF] = Some(SupportedCsrID::pmpcfg15);
    table[0x3B0] = Some(SupportedCsrID::pmpaddr0);
    table[0x3B1] = Some(SupportedCsrID::pmpaddr1);
    table[0x3EF] = Some(SupportedCsrID::pmpaddr63);

    table[0x30C] = Some(SupportedCsrID::mstateen0);
    table[0x30D] = Some(SupportedCsrID::mstateen1);
    table[0x30E] = Some(SupportedCsrID::mstateen2);
    table[0x30F] = Some(SupportedCsrID::mstateen3);
    table[0x31C] = Some(SupportedCsrID::mstateen0h);
    table[0x31D] = Some(SupportedCsrID::mstateen1h);
    table[0x31E] = Some(SupportedCsrID::mstateen2h);
    table[0x31F] = Some(SupportedCsrID::mstateen3h);

    table
};
#[macro_export]
macro_rules! csr {
    ($vm: ident, $csr: ident) => {{
        $vm.cpu.csr($crate::cpu::csr::CsrID::Supported($crate::cpu::csr::SupportedCsrID::$csr))
    }};
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrID {
    Supported(SupportedCsrID),
    Unsupported(u16),
}
impl CsrID {
    pub fn new(id: u16) -> Self {
        assert!(id<(CSR_TABLE.len() as _));
        if let Some(id) = (CSR_TABLE[id as usize]) {
            Self::Supported(id)
        } else {
            Self::Unsupported(id)
        }

    }
    pub fn get(self) -> u16 {
        match self {
            Self::Unsupported(n) => n,
            Self::Supported(n) => n as _
        }
    }
}


#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CsrValue(pub uguest);

impl Display for CsrID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, privilege, description) = match self.get() {
            // Machine Information Registers
            0xF11 => ("mvendorid", "MRO", "Vendor ID."),
            0xF12 => ("marchid", "MRO", "Architecture ID."),
            0xF13 => ("mimpid", "MRO", "Implementation ID."),
            0xF14 => ("mhartid", "MRO", "Hardware thread ID."),
            0xF15 => ("mconfigptr", "MRO", "Pointer to configuration data structure."),

            // Machine Trap Setup
            0x300 => ("mstatus", "MRW", "Machine status register."),
            0x301 => ("misa", "MRW", "ISA and extensions"),
            0x302 => ("medeleg", "MRW", "Machine exception delegation register."),
            0x303 => ("mideleg", "MRW", "Machine interrupt delegation register."),
            0x304 => ("mie", "MRW", "Machine interrupt-enable register."),
            0x305 => ("mtvec", "MRW", "Machine trap-handler base address."),
            0x306 => ("mcounteren", "MRW", "Machine counter enable."),
            0x310 => ("mstatush", "MRW", "Additional machine status register, RV32 only."),
            0x312 => ("medelegh", "MRW", "Upper 32 bits of medeleg, RV32 only."),

            // Machine Trap Handling
            0x340 => ("mscratch", "MRW", "Scratch register for machine trap handlers."),
            0x341 => ("mepc", "MRW", "Machine exception program counter."),
            0x342 => ("mcause", "MRW", "Machine trap cause."),
            0x343 => ("mtval", "MRW", "Machine bad address or instruction."),
            0x344 => ("mip", "MRW", "Machine interrupt pending."),
            0x34A => ("mtinst", "MRW", "Machine trap instruction (transformed)."),
            0x34B => ("mtval2", "MRW", "Machine bad guest physical address."),

            // Machine Configuration
            0x30A => ("menvcfg", "MRW", "Machine environment configuration register."),
            0x31A => ("menvcfgh", "MRW", "Upper 32 bits of menvcfg, RV32 only."),
            0x747 => ("mseccfg", "MRW", "Machine security configuration register."),
            0x757 => ("mseccfgh", "MRW", "Upper 32 bits of mseccfg, RV32 only."),

            // Machine Memory Protection
            0x3A0 => ("pmpcfg0", "MRW", "Physical memory protection configuration."),
            0x3A1 => ("pmpcfg1", "MRW", "Physical memory protection configuration, RV32 only."),
            0x3A2 => ("pmpcfg2", "MRW", "Physical memory protection configuration."),
            0x3A3 => ("pmpcfg3", "MRW", "Physical memory protection configuration, RV32 only."),
            0x3AE => ("pmpcfg14", "MRW", "Physical memory protection configuration."),
            0x3AF => ("pmpcfg15", "MRW", "Physical memory protection configuration, RV32 only."),
            0x3B0 => ("pmpaddr0", "MRW", "Physical memory protection address register."),
            0x3B1 => ("pmpaddr1", "MRW", "Physical memory protection address register."),
            0x3EF => ("pmpaddr63", "MRW", "Physical memory protection address register."),

            // Machine State Enable Registers
            0x30C => ("mstateen0", "MRW", "Machine State Enable 0 Register."),
            0x30D => ("mstateen1", "MRW", "Machine State Enable 1 Register."),
            0x30E => ("mstateen2", "MRW", "Machine State Enable 2 Register."),
            0x30F => ("mstateen3", "MRW", "Machine State Enable 3 Register."),
            0x31C => ("mstateen0h", "MRW", "Upper 32 bits of Machine State Enable 0 Register, RV32 only."),
            0x31D => ("mstateen1h", "MRW", "Upper 32 bits of Machine State Enable 1 Register, RV32 only."),
            0x31E => ("mstateen2h", "MRW", "Upper 32 bits of Machine State Enable 2 Register, RV32 only."),
            0x31F => ("mstateen3h", "MRW", "Upper 32 bits of Machine State Enable 3 Register, RV32 only."),
            0x001 => ("fflags", "URW", "Floating-Point Accrued Exceptions."),
            0x002 => ("frm", "URW", "Floating-Point Dynamic Rounding Mode."),
            0x003 => ("fcsr", "URW", "Floating-Point Control and Status Register (frm +fflags)."),
            
            // Unprivileged Counter/Timers
            0xC00 => ("cycle", "URO", "Cycle counter for RDCYCLE instruction."),
            0xC01 => ("time", "URO", "Timer for RDTIME instruction."),
            0xC02 => ("instret", "URO", "Instructions-retired counter for RDINSTRET instruction."),
            0xC03 => ("hpmcounter3", "URO", "Performance-monitoring counter."),
            0xC04 => ("hpmcounter4", "URO", "Performance-monitoring counter."),
            // ... and so on for other unprivileged CSRs
            
            // Supervisor-level CSRs
            0x100 => ("sstatus", "SRW", "Supervisor status register."),
            0x104 => ("sie", "SRW", "Supervisor interrupt-enable register."),
            0x105 => ("stvec", "SRW", "Supervisor trap handler base address."),
            0x106 => ("scounteren", "SRW", "Supervisor counter enable."),
            0x10A => ("senvcfg", "SRW", "Supervisor environment configuration register."),
            0x120 => ("scountinhibit", "SRW", "Supervisor counter-inhibit register."),
            0x140 => ("sscratch", "SRW", "Scratch register for supervisor trap handlers."),
            0x141 => ("sepc", "SRW", "Supervisor exception program counter."),
            0x142 => ("scause", "SRW", "Supervisor trap cause."),
            0x143 => ("stval", "SRW", "Supervisor bad address or instruction."),
            0x144 => ("sip", "SRW", "Supervisor interrupt pending."),
            0xDA0 => ("scountovf", "SRO", "Supervisor count overflow."),
            0x180 => ("satp", "SRW", "Supervisor address translation and protection."),
            0x5A8 => ("scontext", "SRW", "Supervisor-mode context register."),
            0x10C => ("sstateen0", "SRW", "Supervisor State Enable 0 Register."),
            0x10D => ("sstateen1", "SRW", "Supervisor State Enable 1 Register."),
            0x10E => ("sstateen2", "SRW", "Supervisor State Enable 2 Register."),
            0x10F => ("sstateen3", "SRW", "Supervisor State Enable 3 Register."),
            
            // Hypervisor-level CSRs
            0x600 => ("hstatus", "HRW", "Hypervisor status register."),
            0x602 => ("hedeleg", "HRW", "Hypervisor exception delegation register."),
            0x603 => ("hideleg", "HRW", "Hypervisor interrupt delegation register."),
            0x604 => ("hie", "HRW", "Hypervisor interrupt-enable register."),
            0x606 => ("hcounteren", "HRW", "Hypervisor counter enable."),
            0x607 => ("hgeie", "HRW", "Hypervisor guest external interrupt-enable register."),
            0x612 => ("hedelegh", "HRW", "Upper 32 bits of hedeleg, RV32 only."),
            0x643 => ("htval", "HRW", "Hypervisor bad guest physical address."),
            0x644 => ("hip", "HRW", "Hypervisor interrupt pending."),
            0x645 => ("hvip", "HRW", "Hypervisor virtual interrupt pending."),
            0x64A => ("htinst", "HRW", "Hypervisor trap instruction (transformed)."),
            0xE12 => ("hgeip", "HRO", "Hypervisor guest external interrupt pending."),
            0x60A => ("henvcfg", "HRW", "Hypervisor environment configuration register."),
            0x61A => ("henvcfgh", "HRM", "Upper 32 bits of henvcfg, RV32 only."),
            0x680 => ("hgatp", "HRW", "Hypervisor guest address translation and protection."),
            0x6A8 => ("hcontext", "HRW", "Hypervisor-mode context register."),
            0x605 => ("htimedelta", "HRW", "Delta for VS/VU-mode timer."),
            0x615 => ("htimedeltah", "HRW", "Upper 32 bits of htimedelta, RV32 only."),
            0x60C => ("hstateen0", "HRW", "Hypervisor State Enable 0 Register."),
            0x60D => ("hstateen1", "HRW", "Hypervisor State Enable 1 Register."),
            0x60E => ("hstateen2", "HRW", "Hypervisor State Enable 2 Register."),
            0x60F => ("hstateen3", "HRW", "Hypervisor State Enable 3 Register."),
            0x61C => ("hstateen0h", "HRW", "Upper 32 bits of Hypervisor State Enable 0 Register, RV32 only."),
            0x61D => ("hstateen1h", "HRW", "Upper 32 bits of Hypervisor State Enable 1 Register, RV32 only."),
            0x61E => ("hstateen2h", "HRW", "Upper 32 bits of Hypervisor State Enable 2 Register, RV32 only."),
            0x61F => ("hstateen3h", "HRW", "Upper 32 bits of Hypervisor State Enable 3 Register, RV32 only."),
            
            // Virtual Supervisor Registers
            0x200 => ("vsstatus", "HRW", "Virtual supervisor status register."),
            0x204 => ("vsie", "HRW", "Virtual supervisor interrupt-enable register."),
            0x205 => ("vstvec", "HRW", "Virtual supervisor trap handler base address."),
            0x240 => ("vsscratch", "HRW", "Virtual supervisor scratch register."),
            0x241 => ("vsepc", "HRW", "Virtual supervisor exception program counter."),
            0x242 => ("vscause", "HRW", "Virtual supervisor trap cause."),
            0x243 => ("vstval", "HRW", "Virtual supervisor bad address or instruction."),
            0x244 => ("vsip", "HRW", "Virtual supervisor interrupt pending."),
            0x280 => ("vsatp", "HRW", "Virtual supervisor address translation and protection."),
            
            // Default case for unknown CSR ID
            _ => ("unknown", "unknown", "Unknown CSR ID."),
        };

        write!(f, "{} ({}) - {}", name, privilege, description)
    }
}