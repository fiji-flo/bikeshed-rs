pub fn get_indent_level(text: &str, tab_size: u32) -> u32 {
    let tab_size = tab_size as usize;
    let mut indent_level = 0;
    let mut curr: usize = 0;

    loop {
        if curr >= text.len() {
            break;
        }

        if &text[curr..curr + 1] == "\t" {
            indent_level += 1;
            curr += 1;
        } else if curr + tab_size <= text.len()
            && &text[curr..curr + tab_size] == " ".repeat(tab_size)
        {
            indent_level += 1;
            curr += tab_size;
        } else {
            break;
        }
    }

    indent_level
}

pub fn trim_indent(text: &str, indent_level: u32, tab_size: u32) -> String {
    // Allow empty line.
    if text.trim().is_empty() {
        return text.to_owned();
    }

    let tab_size = tab_size as usize;
    let mut offset: usize = 0;

    for _ in 0..indent_level {
        if &text[offset..offset + 1] == "\t" {
            offset += 1;
        } else if offset + tab_size <= text.len()
            && &text[offset..offset + tab_size] == " ".repeat(tab_size)
        {
            offset += tab_size;
        } else {
            die!("[Markdown] \"{}\" isn't indented enough.", text);
        }
    }

    text[offset..].to_owned()
}
