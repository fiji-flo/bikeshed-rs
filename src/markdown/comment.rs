use crate::line::Line;

pub fn remove_comments(lines: &[Line]) -> Vec<Line> {
    let mut in_comment = false;
    let mut new_lines = Vec::new();

    for line in lines {
        let (new_text, in_comment_now) = remove_comments_in_text(&line.text, in_comment);
        in_comment = in_comment_now;

        if (new_text != line.text && new_text.trim().is_empty())
            || (in_comment && new_text.trim().is_empty())
        {
            continue;
        }

        new_lines.push(Line {
            index: line.index,
            text: new_text.trim_end().to_owned(),
        });
    }

    new_lines
}

fn remove_comments_in_text(text: &str, in_comment: bool) -> (String, bool) {
    if in_comment {
        // [text] or [left, right]
        let pieces = text.splitn(2, "-->").collect::<Vec<&str>>();

        if pieces.len() == 1 {
            // The entire text is a comment.
            ("".to_owned(), true)
        } else {
            // Drop the comment part and check the right part.
            remove_comments_in_text(pieces[1], false)
        }
    } else {
        // [text] or [left, right]
        let pieces = text.splitn(2, "<!--").collect::<Vec<&str>>();

        if pieces.len() == 1 {
            // There aren't any comments in the text.
            (pieces[0].to_owned(), false)
        } else {
            // Keep the non-comment part and check the right part.
            let (rest_text, in_comment_now) = remove_comments_in_text(pieces[1], true);
            (pieces[0].to_owned() + &rest_text, in_comment_now)
        }
    }
}
