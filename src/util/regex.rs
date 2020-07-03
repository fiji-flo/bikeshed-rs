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

#[cfg(test)]
mod tests {
    use super::replace_all;
    use regex::{Captures, Regex};

    #[test]
    fn test_replace_all() {
        {
            let reg = Regex::new(r"-*green-*").unwrap();

            let replacer = |_: &Captures| -> String { "violet".to_owned() };

            assert_eq!(replace_all(&reg, "red", replacer), "red");
            assert_eq!(
                replace_all(&reg, "red green blue", replacer),
                "red violet blue"
            );
            assert_eq!(
                replace_all(&reg, "red ---green--- blue", replacer),
                "red violet blue"
            );
        }
        {
            let reg = Regex::new(r"(-*(?P<color>\w+)-*)").unwrap();

            let replacer = |caps: &Captures| -> String {
                let color = caps["color"].to_lowercase();
                if color == "none" {
                    "violet".to_owned()
                } else {
                    color.to_owned()
                }
            };

            assert_eq!(replace_all(&reg, "red", replacer), "red");
            assert_eq!(
                replace_all(&reg, "red -green- blue", replacer),
                "red green blue"
            );
            assert_eq!(
                replace_all(&reg, "red ---none--- blue", replacer),
                "red violet blue"
            );
        }
    }
}
