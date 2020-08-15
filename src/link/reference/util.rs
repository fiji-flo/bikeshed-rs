use std::collections::HashSet;

pub fn link_text_variations(link_type: &str, link_text: &str) -> HashSet<String> {
    if link_type != "dfn" {
        return hashset! {link_text.to_owned()};
    }

    let mut vars = hashset! {link_text.to_owned()};
    let text = link_text;
    let len = text.len();

    let last1 = if len >= 1 {
        Some(&text[len - 1..])
    } else {
        None
    };

    let last2 = if len >= 2 {
        Some(&text[len - 2..])
    } else {
        None
    };

    let last3 = if len >= 3 {
        Some(&text[len - 3..])
    } else {
        None
    };

    // Example: Navigating <-> Navigate
    if let Some(last3) = last3 {
        if last3 == "ing" {
            vars.insert(text[..len - 3].to_owned());
            vars.insert(format!("{}e", &text[..len - 3]));
        }
    }
    if let Some(last1) = last1 {
        if last1 == "e" {
            vars.insert(format!("{}ing", &text[..len - 1]));
        }
    } else {
        vars.insert(format!("{}ing", text));
    }

    // Example: Snapping <-> Snap
    if let Some(last3) = last3 {
        if last3 == "ing" && len >= 5 && text[len - 4..=len - 4] == text[len - 5..=len - 5] {
            vars.insert(text[..len - 4].to_owned());
        }
    }
    if let Some(last1) = last1 {
        if "bdfgklmnprstvz".contains(last1) {
            vars.insert(format!("{}{}ing", text, last1));
        }
    }

    // Example: Insensitive <-> Insensitively
    if len >= 2 && last2.unwrap() == "ly" {
        vars.insert(text[..len - 2].to_owned());
    } else {
        vars.insert(format!("{}ly", text));
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::link_text_variations;

    #[test]
    fn test_link_text_variations() {
        // Navigating <-> Navigate
        {
            let result = link_text_variations("dfn", "Navigating");
            assert!(result.contains("Navigate"));
        }
        {
            let result = link_text_variations("dfn", "Navigate");
            assert!(result.contains("Navigating"));
        }
        // Snapping <-> Snap
        {
            let result = link_text_variations("dfn", "Snapping");
            assert!(result.contains("Snap"));
        }
        {
            let result = link_text_variations("dfn", "Snap");
            assert!(result.contains("Snapping"));
        }
        // Insensitive <-> Insensitively
        {
            let result = link_text_variations("dfn", "Insensitive");
            assert!(result.contains("Insensitively"));
        }
        {
            let result = link_text_variations("dfn", "Insensitively");
            assert!(result.contains("Insensitive"));
        }
    }
}
