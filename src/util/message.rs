#[macro_export]
macro_rules! die {
    ($($x:expr),+) => ({
        eprintln!($($x),+);
        std::process::exit(1);
    });

    ($($x:expr),+; $line:expr) => ({
        if $line.is_some() {
            eprint!("[Line {}] ", $line.unwrap());
        }
        eprintln!($($x),+);
        std::process::exit(1);
    });
}
