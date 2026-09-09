use crate::{lock::Mutex, sbi::Console};

static WRITER: Mutex<Console> = Mutex::new(Console);

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write as _;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::writer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
