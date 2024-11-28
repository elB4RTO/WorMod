
#[macro_export]
macro_rules! exit_err {
    ($($t:tt),+) => {
        eprintln!("\x1b[91mError:\x1b[0m");
        private_err_descr!($($t),*);
        std::process::exit(1);
    };
}

#[macro_export]
macro_rules! private_err_descr {
    (($($e:expr),+), $t:tt) => {
        eprint!("\x1b[91m→\x1b[0m  ");
        eprintln!($($e),*);
        private_err_descr!($t);
    };
    (($($e:expr),+)) => {
        eprint!("\x1b[91m→\x1b[0m  ");
        eprintln!($($e),*);
    };
}

pub(crate) use {exit_err, private_err_descr};
