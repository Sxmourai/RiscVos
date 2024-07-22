#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}] ", file!(), line!());
    };
    ($val:expr $(, $vals:expr)*) => {
        {
            $crate::println!("[{}:{}] {} = {:?}", file!(), line!(), stringify!($val), &$val);
            // crate::dbg!($($vals),*);
            $( $crate::dbg!($vals); )*
        }
    };
}
#[macro_export]
macro_rules! dbg_bits_reg {
    ($reg: expr) => {{
        crate::println!("{}: {:b}", stringify!($reg), crate::csrr!($reg));
    }};
}
#[macro_export]
macro_rules! dbg_bits {
    ($val:expr $(, $vals:expr)*) => {
        {
            $crate::println!("[{}:{}] {} = {:b}", file!(), line!(), stringify!($val), &$val);
            // crate::dbg!($($vals),*);
            $( $crate::dbg_bits!($vals); )*
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        #[allow(clippy::macro_metavars_in_unsafe)]
        let _ = unsafe{write!($crate::logging::STDIO_UART, $($args)+)};
    });
}

#[macro_export]
macro_rules! println {
    () => ({
        $crate::print!("\r\n")
    });
    ($fmt:expr) => ({
        $crate::print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!($fmt, "\r\n"), $($args)+)
    });
}


pub fn debug_symbols() {
    for i in 0..u8::MAX {
        print!("{}:", i);
        unsafe{STDIO_UART.write_chr(i)};
        println!("!{}", i);
    }
}
pub static mut STDIO_UART: crate::uart::UART = unsafe{crate::uart::UART::new(0x1000_0000)}; 
impl log::Log for crate::uart::UART {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                log::Level::Trace => "\x1b[0;32m",
                log::Level::Debug => "\x1b[0;33m",
                log::Level::Info  => "\x1b[0;34m",
                log::Level::Warn  => "\x1b[0;35m",
                log::Level::Error => "\x1b[0;36m",
            };
            println!("{}{}\x1b[0m: {}", color, record.level(), record.args());
        }
    }

    fn flush(&self) {
        todo!()
    }
}

pub fn init() {
    unsafe { 
        STDIO_UART.init();
        #[allow(static_mut_refs)] // I think deprecated in 2024 version
        log::set_logger(&STDIO_UART)
             .map(|()| log::set_max_level(log::LevelFilter::Info))
    };
}