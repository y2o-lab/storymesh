//! Narrow lexical recognition of custom element registration calls.

#[derive(Debug, PartialEq, Eq)]
enum Token<'a> {
    Ident(&'a [u8]),
    String(&'a [u8]),
    Symbol(u8),
    Other,
}

/// Registrations in source order. `None` means the tag is dynamic or unsupported.
pub(super) fn registrations(source: &[u8]) -> Vec<Option<String>> {
    let tokens = tokenize(source);
    let mut results = Vec::new();
    for i in 0..tokens.len() {
        let is_define = tokens.get(i..i + 4).is_some_and(|part| {
            part == [
                Token::Ident(b"customElements"),
                Token::Symbol(b'.'),
                Token::Ident(b"define"),
                Token::Symbol(b'('),
            ]
        }) && (i == 0 || tokens[i - 1] != Token::Symbol(b'.'));
        let is_decorator = tokens.get(i..i + 3).is_some_and(|part| {
            part == [
                Token::Symbol(b'@'),
                Token::Ident(b"customElement"),
                Token::Symbol(b'('),
            ]
        });
        let (arg, terminator) = if is_define {
            (i + 4, b',')
        } else if is_decorator {
            (i + 3, b')')
        } else {
            continue;
        };
        let tag = match (tokens.get(arg), tokens.get(arg + 1)) {
            (Some(Token::String(value)), Some(Token::Symbol(end))) if *end == terminator => {
                std::str::from_utf8(value)
                    .ok()
                    .filter(|value| valid_tag(value))
                    .map(str::to_owned)
            }
            _ => None,
        };
        // Customized built-ins need `document.createElement(name, { is: tag })`,
        // and cannot use the autonomous-element skeleton.
        results.push(if is_define && has_extra_arguments(&tokens, arg + 2) {
            None
        } else {
            tag
        });
    }
    results
}

fn has_extra_arguments(tokens: &[Token<'_>], start: usize) -> bool {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token {
            Token::Symbol(b'(' | b'[' | b'{') => depth += 1,
            Token::Symbol(b')') if depth == 0 => return false,
            Token::Symbol(b')' | b']' | b'}') => depth = depth.saturating_sub(1),
            Token::Symbol(b',') if depth == 0 => {
                return tokens.get(index + 1) != Some(&Token::Symbol(b')'));
            }
            _ => {}
        }
    }
    true
}

fn valid_tag(tag: &str) -> bool {
    let mut bytes = tag.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && tag.contains('-')
        && tag.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_' | b'.')
        })
        && !matches!(
            tag,
            "annotation-xml"
                | "color-profile"
                | "font-face"
                | "font-face-src"
                | "font-face-uri"
                | "font-face-format"
                | "font-face-name"
                | "missing-glyph"
        )
}

fn tokenize(source: &[u8]) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < source.len() {
        let byte = source[i];
        if byte.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if source[i..].starts_with(b"//") {
            i += 2;
            while i < source.len() && source[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if source[i..].starts_with(b"/*") {
            i += 2;
            while i < source.len() && !source[i..].starts_with(b"*/") {
                i += 1;
            }
            i = (i + 2).min(source.len());
            continue;
        }
        if byte == b'\'' || byte == b'"' || byte == b'`' {
            let quote = byte;
            i += 1;
            let start = i;
            let mut escaped = false;
            while i < source.len() && source[i] != quote {
                if source[i] == b'\\' {
                    escaped = true;
                    i = (i + 2).min(source.len());
                } else {
                    i += 1;
                }
            }
            if i < source.len() && quote != b'`' && !escaped {
                tokens.push(Token::String(&source[start..i]));
            } else {
                tokens.push(Token::Other);
            }
            i = (i + 1).min(source.len());
            continue;
        }
        if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') {
            let start = i;
            i += 1;
            while i < source.len()
                && (source[i].is_ascii_alphanumeric() || matches!(source[i], b'_' | b'$'))
            {
                i += 1;
            }
            tokens.push(Token::Ident(&source[start..i]));
            continue;
        }
        // Regex literals in expression-start positions must not be mistaken for code.
        if byte == b'/'
            && (tokens.is_empty()
                || matches!(
                    tokens.last(),
                    Some(Token::Symbol(
                        b'=' | b'(' | b'[' | b'{' | b',' | b':' | b'!' | b'?' | b';'
                    )) | Some(Token::Ident(b"return" | b"case" | b"throw"))
                ))
        {
            i += 1;
            let mut character_class = false;
            while i < source.len() && source[i] != b'\n' {
                match source[i] {
                    b'\\' => i = (i + 2).min(source.len()),
                    b'[' => {
                        character_class = true;
                        i += 1;
                    }
                    b']' => {
                        character_class = false;
                        i += 1;
                    }
                    b'/' if !character_class => {
                        i += 1;
                        break;
                    }
                    _ => i += 1,
                }
            }
            tokens.push(Token::Other);
            continue;
        }
        tokens.push(Token::Symbol(byte));
        i += 1;
    }
    tokens
}
