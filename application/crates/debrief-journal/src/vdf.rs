//! Minimal Valve KeyValues (VDF/ACF) reader for Steam manifests.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum VdfValue {
    Str(String),
    Table(BTreeMap<String, VdfValue>),
}

impl VdfValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            Self::Table(_) => None,
        }
    }

    pub fn as_table(&self) -> Option<&BTreeMap<String, VdfValue>> {
        match self {
            Self::Table(t) => Some(t),
            Self::Str(_) => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&VdfValue> {
        self.as_table()?.get(key)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }
}

/// Parse a Steam ACF/VDF document into a root table (usually one named key).
pub fn parse_vdf(input: &str) -> Result<BTreeMap<String, VdfValue>, String> {
    let mut lexer = Lexer::new(input);
    let mut root = BTreeMap::new();
    while let Some(tok) = lexer.next_token()? {
        match tok {
            Token::String(key) => {
                let value = parse_value(&mut lexer)?;
                root.insert(key, value);
            }
            Token::Open | Token::Close => {
                return Err("unexpected brace at root".into());
            }
        }
    }
    Ok(root)
}

fn parse_value(lexer: &mut Lexer<'_>) -> Result<VdfValue, String> {
    match lexer.next_token()?.ok_or_else(|| "expected value".to_string())? {
        Token::String(s) => Ok(VdfValue::Str(s)),
        Token::Open => {
            let mut table = BTreeMap::new();
            loop {
                match lexer.next_token()?.ok_or_else(|| "unclosed table".to_string())? {
                    Token::Close => break,
                    Token::String(key) => {
                        let value = parse_value(lexer)?;
                        table.insert(key, value);
                    }
                    Token::Open => return Err("unexpected '{'".into()),
                }
            }
            Ok(VdfValue::Table(table))
        }
        Token::Close => Err("unexpected '}'".into()),
    }
}

#[derive(Debug)]
enum Token {
    String(String),
    Open,
    Close,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        loop {
            match self.chars.peek().copied() {
                None => return Ok(None),
                Some(c) if c.is_whitespace() => {
                    self.chars.next();
                }
                Some('/') => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'/') {
                        while let Some(c) = self.chars.next() {
                            if c == '\n' {
                                break;
                            }
                        }
                    } else {
                        return Err("unexpected '/'".into());
                    }
                }
                Some('{') => {
                    self.chars.next();
                    return Ok(Some(Token::Open));
                }
                Some('}') => {
                    self.chars.next();
                    return Ok(Some(Token::Close));
                }
                Some('"') => return Ok(Some(Token::String(self.read_quoted()?))),
                Some(_) => return Ok(Some(Token::String(self.read_bare()))),
            }
        }
    }

    fn read_quoted(&mut self) -> Result<String, String> {
        self.chars.next(); // opening "
        let mut out = String::new();
        while let Some(c) = self.chars.next() {
            if c == '"' {
                return Ok(out);
            }
            if c == '\\' {
                match self.chars.next() {
                    Some(n) => out.push(n),
                    None => return Err("trailing escape".into()),
                }
            } else {
                out.push(c);
            }
        }
        Err("unclosed string".into())
    }

    fn read_bare(&mut self) -> String {
        let mut out = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
                break;
            }
            out.push(c);
            self.chars.next();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_appmanifest_shape() {
        let text = r#"
"AppState"
{
	"appid"		"424242"
	"name"		"Fixture Quest"
	"installdir"		"Fixture Quest"
}
"#;
        let root = parse_vdf(text).unwrap();
        let app = root.get("AppState").unwrap();
        assert_eq!(app.get_str("appid"), Some("424242"));
        assert_eq!(app.get_str("name"), Some("Fixture Quest"));
    }
}
