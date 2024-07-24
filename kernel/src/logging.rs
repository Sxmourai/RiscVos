use crate::thread::mutex::Mutex;

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
        $crate::println!("{}: {:b}", stringify!($reg), $crate::csrr!($reg));
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
        let _ = unsafe{write!($crate::logging::STDIO_UART.lock(), $($args)+)};
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

pub static mut STDIO_UART: Mutex<crate::uart::UART> =
    Mutex::new(unsafe { crate::uart::UART::new(0x1000_0000) });
pub static _LOGGER: Logger = Logger;
pub struct Logger;
impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                log::Level::Trace => "\x1b[0;32m",
                log::Level::Debug => "\x1b[0;33m",
                log::Level::Info => "\x1b[0;34m",
                log::Level::Warn => "\x1b[0;35m",
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
        STDIO_UART.lock().init();
        // Trace but can be changed using --log-level ... in run.py or any runner script
        #[cfg(debug_assertions)]
        log::set_logger(&_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Trace)).unwrap();
        #[cfg(not(debug_assertions))]
        log::set_logger(&_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info)).unwrap();
    };
}

pub fn handle_int() {
    unsafe { STDIO_UART.lock().handle_int() }
}
