//! Maps this crate's documentation, as the README shows:
//! `cargo run --example docs_map`.
//!
//! The passages are the paragraphs and list items before the references.
//! Headings, tables, code, formulas and link targets are left out, because
//! they are not prose and would give phrases such as "mathrm".

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/concept-map.md".to_string());
    let doc = std::fs::read_to_string(&path).expect("the markdown file can be read");
    let passages = prose(&doc);
    let texts: Vec<&str> = passages.iter().map(String::as_str).collect();
    let map = concept_map::build(&texts).expect("the file has at least 20 passages");
    for community in &map.communities {
        let labels: Vec<&str> = community
            .concepts
            .iter()
            .map(|&id| map.concepts[id as usize].label.as_str())
            .collect();
        println!("- {}", labels.join(", "));
    }
}

/// The paragraphs and list items of `doc` before its references.
fn prose(doc: &str) -> Vec<String> {
    let mut passages = Vec::new();
    let mut in_fence = false;
    for block in doc.split("\n\n") {
        let block = block.trim();
        if block.starts_with("## References") {
            break;
        }
        // A fenced block can hold blank lines, so a fence stays open across blocks.
        let fences = block
            .lines()
            .filter(|line| line.trim_start().starts_with("```"))
            .count();
        if in_fence || fences > 0 {
            in_fence ^= fences % 2 == 1;
            continue;
        }
        let skipped = ["#", "|", "$$"]
            .iter()
            .any(|start| block.starts_with(start));
        if !block.is_empty() && !skipped {
            passages.extend(block.split("\n- ").map(strip));
        }
    }
    passages
}

/// `text` without formulas, code spans, link targets and emphasis marks.
fn strip(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    let skip_to = |chars: &mut std::iter::Peekable<std::str::Chars>, end: char| {
        for c in chars.by_ref() {
            if c == end {
                break;
            }
        }
    };
    while let Some(c) = chars.next() {
        match c {
            '$' => {
                let display = chars.peek() == Some(&'$');
                if display {
                    chars.next();
                }
                skip_to(&mut chars, '$');
                if display {
                    chars.next();
                }
                out.push(' ');
            }
            '`' => {
                skip_to(&mut chars, '`');
                out.push(' ');
            }
            '<' => skip_to(&mut chars, '>'),
            ']' if chars.peek() == Some(&'(') => skip_to(&mut chars, ')'),
            '[' | '*' => {}
            _ => out.push(c),
        }
    }
    out
}
