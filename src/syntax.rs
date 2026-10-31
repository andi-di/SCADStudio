//! Lossless, error-tolerant lexical structure. Never executes OpenSCAD semantics.
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Trivia,
    Comment,
    String,
    Identifier,
    Number,
    Symbol,
    Group,
    Error,
}
#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub range: Range<usize>,
    pub children: Vec<Node>,
}
#[derive(Clone, Debug)]
pub struct Issue {
    pub range: Range<usize>,
    pub message: String,
}
#[derive(Clone, Debug)]
pub struct Syntax {
    pub nodes: Vec<Node>,
    pub issues: Vec<Issue>,
}

impl Syntax {
    /// Every source byte belongs to a leaf or a group delimiter. Invalid input is retained.
    pub fn parse(source: &str) -> Self {
        let mut cursor = 0;
        let mut issues = Vec::new();
        let nodes = sequence(source, &mut cursor, None, &mut issues, 0);
        Self { nodes, issues }
    }

    /// Only a current, complete numeric token can be a literal-edit target.
    pub fn numeric_token(&self, range: &Range<usize>) -> bool {
        fn contains(nodes: &[Node], range: &Range<usize>) -> bool {
            nodes.iter().any(|n| {
                (n.kind == Kind::Number && n.range == *range) || contains(&n.children, range)
            })
        }
        !self
            .issues
            .iter()
            .any(|i| i.range.start < range.end && range.start < i.range.end)
            && contains(&self.nodes, range)
    }
}

fn sequence(
    source: &str,
    cursor: &mut usize,
    close: Option<u8>,
    issues: &mut Vec<Issue>,
    depth: usize,
) -> Vec<Node> {
    if depth >= 128 {
        let range = *cursor..source.len();
        issues.push(Issue {
            range: range.clone(),
            message: "Syntax nesting limit exceeded".into(),
        });
        *cursor = source.len();
        return vec![Node {
            kind: Kind::Error,
            range,
            children: vec![],
        }];
    }
    let opening = cursor.saturating_sub(1);
    let bytes = source.as_bytes();
    let mut nodes: Vec<Node> = Vec::new();
    while *cursor < bytes.len() {
        let start = *cursor;
        let byte = bytes[start];
        if close == Some(byte) {
            *cursor += 1;
            return nodes;
        }
        let mut children = Vec::new();
        let kind;
        if byte.is_ascii_whitespace() {
            *cursor += 1;
            while *cursor < bytes.len() && bytes[*cursor].is_ascii_whitespace() {
                *cursor += 1;
            }
            kind = Kind::Trivia;
        } else if source[start..].starts_with("//") {
            *cursor = source[start..]
                .find('\n')
                .map_or(bytes.len(), |n| start + n);
            kind = Kind::Comment;
        } else if source[start..].starts_with("/*") {
            if let Some(end) = source[start + 2..].find("*/") {
                *cursor = start + end + 4;
                kind = Kind::Comment;
            } else {
                *cursor = bytes.len();
                kind = Kind::Error;
                issues.push(Issue {
                    range: start..*cursor,
                    message: "Unterminated comment".into(),
                });
            }
        } else if byte == b'"' {
            *cursor += 1;
            let mut terminated = false;
            while *cursor < bytes.len() {
                match bytes[*cursor] {
                    b'\\' => *cursor = (*cursor + 2).min(bytes.len()),
                    b'"' => {
                        *cursor += 1;
                        terminated = true;
                        break;
                    }
                    _ => *cursor += 1,
                }
            }
            while !source.is_char_boundary(*cursor) {
                *cursor += 1;
            }
            kind = if terminated {
                Kind::String
            } else {
                issues.push(Issue {
                    range: start..*cursor,
                    message: "Unterminated string".into(),
                });
                Kind::Error
            };
        } else if matches!(byte, b'(' | b'[' | b'{') {
            *cursor += 1;
            let closing = match byte {
                b'(' => b')',
                b'[' => b']',
                _ => b'}',
            };
            children = sequence(source, cursor, Some(closing), issues, depth + 1);
            kind = Kind::Group;
        } else if matches!(byte, b')' | b']' | b'}') {
            *cursor += 1;
            kind = Kind::Error;
            issues.push(Issue {
                range: start..*cursor,
                message: "Unexpected closing delimiter".into(),
            });
        } else if (matches!(byte, b'+' | b'-')
            && bytes
                .get(start + 1)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
            && nodes
                .iter()
                .rev()
                .find(|n| !matches!(n.kind, Kind::Trivia | Kind::Comment))
                .is_none_or(|n| {
                    matches!(
                        source.get(n.range.clone()),
                        Some(
                            "=" | ","
                                | ":"
                                | "?"
                                | "+"
                                | "-"
                                | "*"
                                | "/"
                                | "%"
                                | "^"
                                | "!"
                                | "<"
                                | ">"
                                | "&"
                                | "|"
                        )
                    )
                }))
            || byte.is_ascii_digit()
            || (byte == b'.' && bytes.get(start + 1).is_some_and(u8::is_ascii_digit))
        {
            *cursor += 1;
            while *cursor < bytes.len()
                && (bytes[*cursor].is_ascii_digit() || bytes[*cursor] == b'.')
            {
                *cursor += 1;
            }
            if bytes.get(*cursor).is_some_and(|b| matches!(b, b'e' | b'E')) {
                *cursor += 1;
                if bytes.get(*cursor).is_some_and(|b| matches!(b, b'+' | b'-')) {
                    *cursor += 1;
                }
                while bytes.get(*cursor).is_some_and(u8::is_ascii_digit) {
                    *cursor += 1;
                }
            }
            kind = if source[start..*cursor].parse::<f64>().is_ok() {
                Kind::Number
            } else {
                Kind::Error
            };
        } else {
            let ch = source[start..].chars().next().unwrap_or('\0');
            *cursor += ch.len_utf8();
            if ch.is_alphabetic() || matches!(ch, '_' | '$') {
                while let Some(ch) = source[*cursor..].chars().next() {
                    if !ch.is_alphanumeric() && !matches!(ch, '_' | '$') {
                        break;
                    }
                    *cursor += ch.len_utf8();
                }
                kind = Kind::Identifier;
            } else {
                kind = Kind::Symbol;
            }
        }
        if kind == Kind::Error
            && !issues
                .iter()
                .any(|i| i.range.start <= start && i.range.end >= *cursor)
        {
            issues.push(Issue {
                range: start..*cursor,
                message: "Invalid token".into(),
            });
        }
        nodes.push(Node {
            kind,
            range: start..*cursor,
            children,
        });
    }
    if let Some(closing) = close {
        issues.push(Issue {
            range: opening..*cursor,
            message: format!("Missing {}", closing as char),
        });
    }
    nodes
}

/// A statement envelope points into the same lossless source; no formatting is synthesized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatementKind {
    Assignment,
    Module,
    Function,
    Include,
    Use,
    For,
    If,
    Let,
    Call,
    Block,
    Unknown,
}
#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub range: Range<usize>,
    pub children: Vec<Statement>,
}
impl Syntax {
    /// Coarse program navigation, including incomplete statement envelopes.
    /// Detailed expression/operator semantics remain the original parser's responsibility.
    pub fn statements(&self, source: &str) -> Vec<Statement> {
        fn build(nodes: &[Node], source: &str) -> Vec<Statement> {
            let mut result = Vec::new();
            let mut pending = Vec::new();
            for node in nodes {
                pending.push(node);
                if source.get(node.range.clone()) == Some(";")
                    || (node.kind == Kind::Group
                        && source.as_bytes().get(node.range.start) == Some(&b'{'))
                {
                    append(&mut result, &mut pending, source);
                }
            }
            append(&mut result, &mut pending, source);
            result
        }
        fn append(result: &mut Vec<Statement>, pending: &mut Vec<&Node>, source: &str) {
            let significant: Vec<_> = pending
                .iter()
                .filter(|n| !matches!(n.kind, Kind::Trivia | Kind::Comment))
                .collect();
            if let (Some(first), Some(last)) = (significant.first(), significant.last()) {
                let word = source.get(first.range.clone()).unwrap_or_default();
                let kind = match word {
                    "module" => StatementKind::Module,
                    "function" => StatementKind::Function,
                    "include" => StatementKind::Include,
                    "use" => StatementKind::Use,
                    "for" | "intersection_for" => StatementKind::For,
                    "if" => StatementKind::If,
                    "let" | "assign" => StatementKind::Let,
                    _ if first.kind == Kind::Group => StatementKind::Block,
                    _ if significant
                        .get(1)
                        .is_some_and(|n| source.get(n.range.clone()) == Some("=")) =>
                    {
                        StatementKind::Assignment
                    }
                    _ if first.kind == Kind::Identifier => StatementKind::Call,
                    _ => StatementKind::Unknown,
                };
                let children = pending
                    .iter()
                    .filter(|n| {
                        n.kind == Kind::Group && source.as_bytes().get(n.range.start) == Some(&b'{')
                    })
                    .flat_map(|n| build(&n.children, source))
                    .collect();
                result.push(Statement {
                    kind,
                    range: first.range.start..last.range.end,
                    children,
                });
            }
            pending.clear();
        }
        build(&self.nodes, source)
    }
}
