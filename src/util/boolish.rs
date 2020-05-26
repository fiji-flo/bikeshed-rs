pub fn boolish_to_bool(val: &str) -> Option<bool> {
    match &*val.to_lowercase() {
        "true" | "yes" | "y" | "on" => Some(true),
        "false" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}
