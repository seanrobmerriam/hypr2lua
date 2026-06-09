//! Parser for Hyprland hyprlang `.conf` files.
//!
//! This module provides a line-oriented recursive descent parser that converts
//! hyprlang configuration text into an abstract syntax tree (AST) of [`Node`] values.

/// A node in the hyprlang AST.
///
/// Each variant represents a different syntactic element found in `.conf` files.
///
/// # Examples
///
/// ```
/// use hypr2lua::parser::Node;
///
/// let node = Node::Variable {
///     name: "mod".to_string(),
///     value: "SUPER".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub enum Node {
    /// A comment line (text after `#`).
    Comment(String),
    /// An empty line.
    BlankLine,
    /// A variable assignment (`$name = value`).
    Variable {
        /// The variable name (without the leading `$`).
        name: String,
        /// The variable value.
        value: String,
    },
    /// A key-value assignment (`key = value`).
    Assignment {
        /// The assignment key.
        key: String,
        /// The assignment value.
        value: String,
    },
    /// A section block (`name { ... }`).
    Section {
        /// The section name.
        name: String,
        /// Child nodes within the section.
        children: Vec<Node>,
    },
    /// A bind directive (`bind`, `bindl`, `bindd`, etc.).
    Bind {
        /// The bind variant (e.g., "bind", "bindl", "bindd").
        variant: String,
        /// The bind arguments.
        args: Vec<String>,
    },
    /// A keyword directive (`exec-once`, `windowrule`, `monitor`, etc.).
    Keyword {
        /// The keyword name.
        name: String,
        /// The keyword arguments.
        args: Vec<String>,
    },
    /// A source directive (`source = path`).
    Source {
        /// The path to source.
        path: String,
    },
}

/// Parses hyprlang configuration text into an AST.
///
/// # Arguments
///
/// * `input` - The hyprlang `.conf` file contents as a string.
///
/// # Returns
///
/// A vector of [`Node`] values representing the parsed configuration.
///
/// # Examples
///
/// ```
/// use hypr2lua::parser::{parse, Node};
///
/// let input = "$mod = SUPER\nbind = $mod, Q, exec, terminal";
/// let nodes = parse(input);
/// assert_eq!(nodes.len(), 3); // variable, blank, bind
/// ```
pub fn parse(input: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            nodes.push(Node::BlankLine);
            i += 1;
            continue;
        }

        if trimmed.starts_with('#') {
            nodes.push(Node::Comment(trimmed[1..].to_string()));
            i += 1;
            continue;
        }

        if trimmed.starts_with("source") && trimmed[6..].trim_start().starts_with('=') {
            let path = trimmed.split_once('=').unwrap().1.trim().to_string();
            nodes.push(Node::Source { path });
            i += 1;
            continue;
        }

        if trimmed.starts_with("source ") || trimmed.starts_with("source\t") {
            let path = trimmed[6..].trim().to_string();
            nodes.push(Node::Source { path });
            i += 1;
            continue;
        }

        if trimmed.ends_with('{') {
            let name = trimmed.trim_end_matches('{').trim().to_string();
            let mut children = Vec::new();
            i += 1;
            let mut depth = 1u32;
            while i < lines.len() && depth > 0 {
                let inner = lines[i].trim();
                if inner.ends_with('{') {
                    depth += 1;
                    let child_name = inner.trim_end_matches('{').trim().to_string();
                    let mut grandchildren = Vec::new();
                    i += 1;
                    let mut inner_depth = 1u32;
                    while i < lines.len() && inner_depth > 0 {
                        let gc_line = lines[i].trim();
                        if gc_line.ends_with('{') {
                            inner_depth += 1;
                        }
                        if gc_line == "}" {
                            inner_depth -= 1;
                            if inner_depth == 0 {
                                i += 1;
                                break;
                            }
                        }
                        if inner_depth == 1 {
                            grandchildren.push(parse_line(gc_line));
                        }
                        i += 1;
                    }
                    children.push(Node::Section { name: child_name, children: grandchildren });
                    depth -= 1;
                    continue;
                }
                if inner == "}" {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                children.push(parse_line(inner));
                i += 1;
            }
            nodes.push(Node::Section { name, children });
            continue;
        }

        nodes.push(parse_line(trimmed));
        i += 1;
    }

    nodes
}

fn parse_line(line: &str) -> Node {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Node::BlankLine;
    }

    if trimmed.starts_with('#') {
        return Node::Comment(trimmed[1..].to_string());
    }

    if trimmed.starts_with('$') {
        if let Some((name, value)) = trimmed[1..].split_once('=') {
            return Node::Variable {
                name: name.trim().to_string(),
                value: value.trim().to_string(),
            };
        }
    }

    let bind_variants = ["bindeld", "bindld", "binddr", "bindmd", "bindle", "bindlr", "bindd", "bindr", "bindl", "bindm", "bind"];
    for variant in &bind_variants {
        if trimmed.starts_with(variant) && trimmed[variant.len()..].trim_start().starts_with('=') {
            let args_str = trimmed.split_once('=').unwrap().1.trim();
            let args = split_args(args_str);
            return Node::Bind {
                variant: variant.to_string(),
                args,
            };
        }
    }

    let keywords = [
        "exec-once", "exec", "windowrulev2", "windowrule", "workspace",
        "monitor", "bezier", "animation", "layerrule", "plugin", "env", "unbind",
    ];
    for kw in &keywords {
        if trimmed.starts_with(kw) && trimmed[kw.len()..].trim_start().starts_with('=') {
            let args_str = trimmed.split_once('=').unwrap().1.trim();
            let args = split_args(args_str);
            return Node::Keyword {
                name: kw.to_string(),
                args,
            };
        }
    }

    if let Some((key, value)) = trimmed.split_once('=') {
        return Node::Assignment {
            key: key.trim().to_string(),
            value: value.trim().to_string(),
        };
    }

    Node::Comment(format!(" UNPARSED: {}", trimmed))
}

fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0u32;

    for ch in s.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                args.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let last = current.trim().to_string();
    if !last.is_empty() {
        args.push(last);
    }

    args
}
