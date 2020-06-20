pub fn boolish_to_bool(val: &str) -> Result<bool, &'static str> {
    match &*val.to_lowercase() {
        "true" | "yes" | "y" | "on" => Ok(true),
        "false" | "no" | "n" | "off" => Ok(false),
        _ => Err("the input is not boolish"),
    }
}
