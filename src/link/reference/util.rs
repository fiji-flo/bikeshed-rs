use std::collections::HashSet;

pub fn link_text_variations(link_type: &str, link_text: &str) -> HashSet<String> {
    if link_type != "dfn" {
        return hashset! {link_text.to_owned()};
    }

    let mut vars = hashset! {link_text.to_owned()};
    let text = link_text;
    let len = text.len();

    let last1 = || -> &str { &text[len - 1..] };
    let last2 = || -> &str { &text[len - 2..] };
    let last3 = || -> &str { &text[len - 3..] };

    // Example: Berries <-> Berry
    if len >= 3 && last3() == "ies" {
        vars.insert(format!("{}y", &text[..len - 3]));
    }
    if len >= 1 && last1() == "y" {
        vars.insert(format!("{}ies", &text[..len - 1]));
    }

    // Example: Blockified <-> Blockify
    if len >= 3 && last3() == "ied" {
        vars.insert(format!("{}y", &text[..len - 3]));
    }
    if len >= 1 && last1() == "y" {
        vars.insert(format!("{}ied", &text[..len - 1]));
    }

    // Example: Zeroes <-> Zero
    if len >= 2 && last2() == "es" {
        vars.insert(text[..len - 2].to_owned());
    } else {
        vars.insert(format!("{}es", &text));
    }

    // Example: Bikeshed's <-> Bikeshed
    if len >= 2 && (last2() == "'s" || last2() == "’s") {
        vars.insert(text[..len - 2].to_owned());
    } else {
        vars.insert(format!("{}'s", &text));
    }

    // Example: Bikesheds <-> Bikeshed
    if len >= 1 && last1() == "s" {
        vars.insert(text[..len - 1].to_owned());
    } else {
        vars.insert(format!("{}s", &text));
    }

    // Example: Bikesheds <-> Bikesheds'
    if len >= 1 && (last1() == "'" || last1() == "’") {
        vars.insert(text[..len - 1].to_owned());
    } else {
        vars.insert(format!("{}'", &text));
    }

    // Example: Snapped <-> Snap
    if len >= 4 && last2() == "ed" && text[len - 3..=len - 3] == text[len - 4..=len - 4] {
        vars.insert(text[..len - 3].to_owned());
    } else if "bdfgklmnprstvz".contains(last1()) {
        vars.insert(format!("{}{}ed", &text, last1()));
    }

    // Example: Zeroed <-> Zero
    if len >= 2 && last2() == "ed" {
        vars.insert(text[..len - 2].to_owned());
    } else {
        vars.insert(format!("{}ed", &text));
    }

    // Example: Generated <-> Generate
    if len >= 1 && last1() == "d" {
        vars.insert(text[..len - 1].to_owned());
    } else {
        vars.insert(format!("{}d", &text));
    }

    // Example: Navigating <-> Navigate
    if len >= 3 && last3() == "ing" {
        vars.insert(text[..len - 3].to_owned());
        vars.insert(format!("{}e", &text[..len - 3]));
    } else if len >= 1 && last1() == "e" {
        vars.insert(format!("{}ing", &text[..len - 1]));
    } else {
        vars.insert(format!("{}ing", text));
    }

    // Example: Snapping <-> Snap
    if len >= 5 && last3() == "ing" && text[len - 4..=len - 4] == text[len - 5..=len - 5] {
        vars.insert(text[..len - 4].to_owned());
    } else if len >= 1 && "bdfgklmnprstvz".contains(last1()) {
        vars.insert(format!("{}{}ing", text, last1()));
    }

    // Example: Insensitive <-> Insensitively
    if len >= 2 && last2() == "ly" {
        vars.insert(text[..len - 2].to_owned());
    } else {
        vars.insert(format!("{}ly", text));
    }

    // Special irregular case: throw <-> thrown
    if text == "throw" {
        vars.insert("thrown".to_owned());
    }
    if text == "thrown" {
        vars.insert("throw".to_owned());
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::link_text_variations;

    #[test]
    fn test_link_text_variations() {
        let cases = hashset! {
            // Berries <-> Berry
            ("Berries", "Berry"),
            // Blockified <-> Blockified
            ("Blockified", "Blockified"),
            // Zeroes <-> Zero
            ("Zeroes", "Zero"),
            // Bikeshed's <-> Bikeshed
            ("Bikeshed's", "Bikeshed"),
            // Bikesheds <-> Bikeshed
            ("Bikesheds", "Bikeshed"),
            // Bikesheds <-> Bikesheds'
            ("Bikesheds", "Bikesheds'"),
            // Snapped <-> Snap
            ("Snapped", "Snap"),
            // Zeroed <-> Zero
            ("Zeroed", "Zero"),
            // Generated <-> Generate
            ("Generated", "Generate"),
            // Navigating <-> Navigate
            ("Navigating", "Navigating"),
            // Snapping <-> Snap
            ("Snapping", "Snap"),
            // Insensitive <-> Insensitively
            ("Insensitive", "Insensitively"),
            // throw <-> thrown
            ("throw", "thrown"),
        };

        for (lhs, rhs) in cases.into_iter() {
            {
                let result = link_text_variations("dfn", lhs);
                assert!(result.contains(rhs));
            }
            {
                let result = link_text_variations("dfn", rhs);
                assert!(result.contains(lhs));
            }
        }
    }
}
