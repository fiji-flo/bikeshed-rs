use std::collections::HashSet;

pub fn boolish_to_bool(val: &str) -> Option<bool> {
    lazy_static! {
        pub static ref TRUE_VALUES: HashSet<&'static str> = {
            hashset! {"true", "yes", "y", "on"}
        };
        pub static ref FALSE_VALUES: HashSet<&'static str> = {
            hashset! {"false", "no", "n", "off"}
        };
    }

    let val = val.to_lowercase();

    if TRUE_VALUES.contains(&val.as_ref()) {
        Some(true)
    } else if FALSE_VALUES.contains(&val.as_ref()) {
        Some(false)
    } else {
        None
    }
}
