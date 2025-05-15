pub mod php;

trait CamelCase {
    fn to_camel_case(&self) -> String;
}

fn to_camel_case(s: &str) -> String {
    let mut parts = s
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty());

    let mut result = String::new();

    if let Some(first) = parts.next() {
        result.push_str(&first.to_lowercase());
    }

    for part in parts {
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            result.push(first_char.to_ascii_uppercase());
            result.push_str(&chars.as_str().to_ascii_lowercase());
        }
    }

    result
}

impl CamelCase for str {
    fn to_camel_case(&self) -> String {
        to_camel_case(self)
    }
}

trait PascalCase {
    fn to_pascal_case(&self) -> String;
}

fn to_pascal_case(s: &str) -> String {
    let parts = s
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty());

    let mut result = String::new();

    for part in parts {
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            result.push(first_char.to_ascii_uppercase());
            result.push_str(&chars.as_str().to_ascii_lowercase());
        }
    }

    result
}

impl PascalCase for str {
    fn to_pascal_case(&self) -> String {
        to_pascal_case(self)
    }
}
