
#[macro_export]
macro_rules! print_err {
    ($($x:expr),*) => {
        eprint!("WorMod: Error: ");
        eprintln!($($x),*);
    };
}

pub(crate) use print_err;
