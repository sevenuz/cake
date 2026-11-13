//! the markdown cake bodies use, as styled lines: headings, **bold**,
//! *italic*, `code`, fenced code, lists, > quotes, rules and [[links]].
//! Used by `show` (printed) and the TUI (ratatui).

use crate::{style, theme};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// styled lines of `text`; rules are `width` wide
pub fn render(text: &str, width: u16) -> Vec<Line<'static>> {
    let t = theme::t();
    let g = style::g();
    let mut out = vec![];
    let mut fence = false;
    for raw in text.lines() {
        let trimmed = raw.trim_start();
        if trimmed.starts_with("```") {
            fence = !fence;
            continue;
        }
        if fence {
            out.push(Line::from(Span::styled(format!("  {}", raw), Style::default().fg(t.code))));
            continue;
        }
        let indent = &raw[..raw.len() - trimmed.len()];
        if let Some((level, rest)) = heading(trimmed) {
            let mut s = Style::default().fg(t.heading).add_modifier(Modifier::BOLD);
            if level == 1 {
                s = s.add_modifier(Modifier::UNDERLINED);
            }
            out.push(Line::from(Span::styled(rest.to_string(), s)));
        } else if is_rule(trimmed) {
            out.push(Line::from(Span::styled(
                g.rule.to_string().repeat(width.clamp(3, 80) as usize),
                Style::default().fg(t.dim),
            )));
        } else if let Some(rest) = trimmed.strip_prefix("> ").or(trimmed.strip_prefix(">").filter(|r| r.is_empty())) {
            let mut spans = vec![Span::styled(format!("{}{}", indent, g.pipe), Style::default().fg(t.quote))];
            spans.extend(inline(rest, Style::default().fg(t.quote)));
            out.push(Line::from(spans));
        } else if let Some(rest) = ["- ", "* ", "+ "].iter().find_map(|b| trimmed.strip_prefix(b)) {
            let mut spans = vec![Span::raw(format!("{}{} ", indent, g.bullet))];
            spans.extend(inline(rest, Style::default()));
            out.push(Line::from(spans));
        } else {
            let mut spans = vec![Span::raw(indent.to_string())];
            spans.extend(inline(trimmed, Style::default()));
            out.push(Line::from(spans));
        }
    }
    out
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&level).then(|| line[level..].strip_prefix(' ')).flatten().map(|r| (level, r))
}

fn is_rule(line: &str) -> bool {
    let l = line.trim();
    l.len() >= 3 && ["-", "*", "_"].iter().any(|c| l.chars().all(|x| x.to_string() == *c))
}

/// **bold**, *italic* / _italic_, `code`, [[link]] within one line
fn inline(text: &str, base: Style) -> Vec<Span<'static>> {
    let t = theme::t();
    let mut spans = vec![];
    let mut plain = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let flush = |plain: &mut String, spans: &mut Vec<Span<'static>>| {
        if !plain.is_empty() {
            spans.push(Span::styled(std::mem::take(plain), base));
        }
    };
    let find = |from: usize, pat: &[char]| -> Option<usize> {
        (from..chars.len().saturating_sub(pat.len() - 1)).find(|&j| chars[j..j + pat.len()] == *pat)
    };
    while i < chars.len() {
        let rest_is = |p: &str| chars[i..].iter().take(p.len()).copied().eq(p.chars());
        let marked = if rest_is("**") {
            find(i + 2, &['*', '*']).map(|end| (i + 2, end, 2, base.fg(t.bold).add_modifier(Modifier::BOLD)))
        } else if rest_is("[[") {
            find(i + 2, &[']', ']']).map(|end| (i + 2, end, 2, base.fg(t.hint)))
        } else if chars[i] == '`' {
            find(i + 1, &['`']).map(|end| (i + 1, end, 1, base.fg(t.code)))
        } else if (chars[i] == '*' || chars[i] == '_')
            && (i == 0 || !chars[i - 1].is_alphanumeric())
            && chars.get(i + 1).is_some_and(|c| !c.is_whitespace())
        {
            find(i + 1, &[chars[i]]).map(|end| (i + 1, end, 1, base.add_modifier(Modifier::ITALIC)))
        } else {
            None
        };
        match marked {
            Some((start, end, len, s)) if end > start => {
                flush(&mut plain, &mut spans);
                spans.push(Span::styled(chars[start..end].iter().collect::<String>(), s));
                i = end + len;
            }
            _ => {
                plain.push(chars[i]);
                i += 1;
            }
        }
    }
    flush(&mut plain, &mut spans);
    spans
}

/// a line as ANSI text for printed output
pub fn ansi(line: &Line) -> String {
    line.spans.iter().map(|s| theme::paint(&s.content, s.style)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &str) -> Vec<String> {
        render(text, 10).iter().map(|l| l.spans.iter().map(|s| s.content.to_string()).collect()).collect()
    }

    #[test]
    fn markup_is_removed_and_structure_kept() {
        let text = "# Title\n\nsome **bold** and *it* with `code` and [[id]]\n- item\n  - sub\n> quote\n---\n```\nlet x = *y;\n```";
        let lines = plain(text);
        assert_eq!(lines[0], "Title");
        assert_eq!(lines[2], "some bold and it with code and id");
        assert!(lines[3].ends_with(" item") && lines[4].starts_with("  ") && lines[4].ends_with(" sub"));
        assert!(lines[5].ends_with("quote"));
        assert_eq!(lines[6].chars().count(), 10);
        assert_eq!(lines[7], "  let x = *y;");
    }

    #[test]
    fn lone_markers_stay_text() {
        assert_eq!(plain("2 * 3 = 6, snake_case, a ** b")[0], "2 * 3 = 6, snake_case, a ** b");
    }
}
