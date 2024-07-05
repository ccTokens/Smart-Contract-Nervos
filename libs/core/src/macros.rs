#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! cc_assert {
    ($condition:expr, $error:expr) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!("{}", alloc::string::ToString::to_string(&$error)));
            return core::result::Result::Err($error.into());
        }
    };
}
