pub fn gen_doc_string_opt(string: &Option<String>) -> String {
    string
        .as_ref()
        .map(|s| gen_doc_string(s))
        .unwrap_or_default()
}

fn trim_lines(string: &str) -> String {
    let lines = string.split('\n');
    let num_leading = lines
        .filter_map(|l| l.chars().position(|c| !c.is_whitespace()))
        .min()
        .unwrap_or(0);
    string
        .split('\n')
        .map(|line| {
            if line.len() > num_leading {
                &line[num_leading..]
            } else {
                line
            }
        })
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn gen_doc_string(string: &str) -> String {
    let lines = string.lines();
    if lines.count() == 1 {
        format!("/** {} */\n", string.trim())
    } else {
        format!(
            "/**\n{}\n */\n",
            trim_lines(string)
                .lines()
                .map(|line| format!(" * {}", line))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

pub fn indent(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("  {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
