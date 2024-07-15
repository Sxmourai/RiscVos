// use spin::Mutex;

use crate::uart::UART;
// TODO: Use Mutex
pub static mut STDIO_UART: UART = unsafe {UART::new(0x1000_0000)};

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
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = unsafe{write!($crate::console::STDIO_UART, $($args)+)};
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