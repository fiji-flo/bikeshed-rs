use regex::{Captures, Regex};

pub fn replace_all(reg: &Regex, haystack: &str, replacer: impl Fn(&Captures) -> String) -> String {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;

    for caps in reg.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacer(&caps));
        last_match = m.end();
    }

    new.push_str(&haystack[last_match..]);
    new
}
