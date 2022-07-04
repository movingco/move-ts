pub fn gen_doc_string_opt(string: &Option<String>) -> String {
    string
        .as_ref()
        .map(|s| gen_doc_string(s))
        .unwrap_or_default()
}

pub fn gen_doc_string(string: &str) -> String {
    let lines = string.lines();
    if lines.count() == 1 {
        format!("/** {} */", string.trim())
    } else {
        format!(
            "/**\n{}\n */",
            string
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
