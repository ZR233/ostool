// #[cfg(feature = "logging")]
// macro_rules! debug {
//     ($($arg:tt)*) => {
//         log::debug!($($arg)*);
//     };
// }

// #[cfg(not(feature = "logging"))]
// macro_rules! debug {
//     ($($arg:tt)*) => {};
// }

#[cfg(feature = "logging")]
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

#[cfg(not(feature = "logging"))]
macro_rules! info {
    ($($arg:tt)*) => {};
}

#[cfg(feature = "logging")]
macro_rules! warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}

#[cfg(not(feature = "logging"))]
macro_rules! warn {
    ($($arg:tt)*) => {};
}
