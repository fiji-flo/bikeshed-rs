#[macro_export]
macro_rules! die {
    ($($x:expr),+) => ({
        eprintln!($($x),+);
        std::process::exit(1);
    });

    ($($x:expr),+; $line:expr) => ({
        if let Some(line) = $line {
            eprint!("[Line {}] ",line);
        }
        eprintln!($($x),+);
        std::process::exit(1);
    });
}
