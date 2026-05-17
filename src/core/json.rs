pub fn split_json_objects(input: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0
                    && let Some(s) = start
                {
                    objects.push(input[s..=i].to_string());
                    start = None;
                }
            }
            '"' => {
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 1;
                    } else if chars[i] == '"' {
                        break;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    objects
}

pub fn extract_json_str(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = obj.find(&pattern)?;
    let after = obj[idx + pattern.len()..].trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    if !after.starts_with('"') {
        return None;
    }
    let mut result = String::new();
    let mut chars = after[1..].chars();
    loop {
        match chars.next()? {
            '\\' => match chars.next()? {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'n' => result.push('\n'),
                c => {
                    result.push('\\');
                    result.push(c);
                }
            },
            '"' => break,
            c => result.push(c),
        }
    }
    Some(result)
}
