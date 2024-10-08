/// TODO
pub mod lexer_state;
use crate::prelude::*;

macro_rules! tmp_lexer_builder {
    (
        DefaultSystem {
            number: $number:literal,
            symbol: $symbol:literal,
            keyword: $keyword:literal,
            whitespace: {
                allow_them: $allow_whitespace:literal,
                use_system: $whitespace:literal$(,)?
            }$(,)?
        },
        Symbols {
            $($sym:literal => $variant:ident),+ $(,)?
        },
        Keyword {
            $($x:literal),* $(,)?
        },
        Number {
            trailing: $trailing:literal,
            float: $float:literal,
            u_int: $u_int:literal,
            int: $int:literal $(,)?
        }$(,)?
    ) => {
        crate::keywords!($($x,)*);
        crate::symbols!($($sym => $variant,)*);
        pub type System = fn(char, &mut LexerState) -> Option<Token>;
        #[derive(Debug, Default)]
        pub struct AtlasLexer {
            sys: Vec<System>,
            path: &'static str,
            pub current_pos: BytePos,
            pub source: String,
        }
        impl AtlasLexer {
            pub fn default() -> Self {
                let mut lexer = AtlasLexer::new("<stdin>", String::new());
                if $number {lexer.add_system(default_number);}
                if $symbol {lexer.add_system(default_symbol);}
                if $keyword {lexer.add_system(default_keyword);}
                if $whitespace {lexer.add_system(default_whitespace);}
                lexer
            }
            pub fn new(path: &'static str, source: String) -> Self {
                Self {
                    sys: vec![],
                    path,
                    current_pos: BytePos::from(0),
                    source,
                }
            }

            pub fn set_source(&mut self, source: String) -> &mut Self {
                self.source = source;
                self
            }

            pub fn set_path(&mut self, new_path: &'static str) -> &mut Self {
                self.path = new_path;
                self
            }

            pub fn add_system(
                &mut self,
                s: fn(char, &mut LexerState) -> Option<Token>,
            ) -> &mut Self {
                self.sys.push(s);
                self
            }

            //A way of handling errors will come later
            pub fn tokenize(&mut self) -> Result<Vec<Token>, ()> {
                let mut tok: Vec<Token> = vec![];
                tok.push(Token::new(
                    Span {
                        start: self.current_pos,
                        end: self.current_pos,
                        path: self.path,
                    },
                    TokenKind::SoI,
                ));
                loop {
                    let ch = self.source.chars().nth(usize::from(self.current_pos));
                    match ch {
                        Some(c) => {
                            let state = LexerState::new(
                                self.current_pos,
                                self.source
                                    .get(usize::from(self.current_pos)..self.source.len())
                                    .unwrap(),
                                self.path,
                            );
                            let mut counter = 0;
                            for f in &self.sys {
                                let mut current_state = state.clone();
                                match f(c, &mut current_state) {
                                    Some(f) => {
                                        if !$allow_whitespace {
                                            match f.kind() {
                                                TokenKind::WhiteSpace => {}
                                                TokenKind::CarriageReturn => {}
                                                TokenKind::NewLine => {}
                                                TokenKind::Tabulation => {}
                                                _ => tok.push(f),
                                            }
                                        } else {
                                            tok.push(f);
                                        }
                                        self.current_pos = current_state.current_pos;
                                        break;
                                    }
                                    None => {
                                        counter += 1;
                                        continue;
                                    }
                                }
                            }
                            if counter >= self.sys.len() {
                                return Err(());
                            }
                        }
                        None => break,
                    }
                }
                tok.push(Token::new(
                    Span {
                        start: self.current_pos,
                        end: self.current_pos,
                        path: self.path,
                    },
                    TokenKind::EoI,
                ));
                return Ok(tok);
            }
        }
        const TRAILING: [&str; 10] = if $trailing { ["_u8", "_u16", "_u32", "_u64", "_i8", "_i16", "_i32", "_i64", "_f32", "_f64"] } else { [""; 10] };
        fn default_number(c: char, state: &mut LexerState) -> Option<Token> {
            if c.is_numeric() {
                let start = state.current_pos;
                let mut is_float = false;
                let mut n = String::new();
                n.push(c);
                state.next();
                loop {
                    if let Some(c) = state.peek() {
                        if c.is_numeric() {
                            n.push(*c);
                            state.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if let Some(&'.') = state.peek() {
                        n.push('.');
                        state.next();
                        is_float = true;
                        loop {
                            if let Some(c) = state.peek() {
                                if c.is_numeric() {
                                    n.push(*c);
                                    state.next();
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                }

                Some(Token::new(
                    Span {
                        start,
                        end: state.current_pos,
                        path: state.path,
                    },
                    TokenKind::Literal(Literal::Float(n.parse::<f64>().unwrap())),
                ))
            } else {
                None
            }
        }
        fn default_whitespace(c: char, state: &mut LexerState) -> Option<Token> {
            let start = state.current_pos;
            let tok = match c {
                ' ' => TokenKind::WhiteSpace,
                '\t' => TokenKind::Tabulation,
                '\n' => TokenKind::NewLine,
                '\r' => TokenKind::CarriageReturn,
                _ => return None,
            };
            state.next();
            return Some(Token::new(
                Span {
                    start,
                    end: state.current_pos,
                    path: state.path,
                },
                tok,
            ))
        }

    };
}

/// To be done
#[macro_export]
macro_rules! lexer_builder {
    (ignore_space: $ignore:literal) => {
        #[derive(Debug, Default)]
        pub struct AtlasLexer {
            sys: Vec<fn(char, &mut LexerState) -> Option<Token>>,
            path: &'static str,
            pub current_pos: BytePos,
            pub source: String,
        }
        impl AtlasLexer {
            //The default should only add the existing default system => find a way to do it correctly :thinking:
            pub fn default() -> Self {
                let mut lexer = AtlasLexer::new("<stdin>", String::new());
                lexer
                    .add_system(default_number)
                    .add_system(default_symbol)
                    .add_system(default_keyword)
                    .add_system(default_whitespace);
                lexer
            }
            pub fn new(path: &'static str, source: String) -> Self {
                Self {
                    sys: vec![],
                    path,
                    current_pos: BytePos::from(0),
                    source,
                }
            }

            pub fn set_source(&mut self, source: String) -> &mut Self {
                self.source = source;
                self
            }

            pub fn set_path(&mut self, new_path: &'static str) -> &mut Self {
                self.path = new_path;
                self
            }

            pub fn add_system(
                &mut self,
                s: fn(char, &mut LexerState) -> Option<Token>,
            ) -> &mut Self {
                self.sys.push(s);
                self
            }

            //A way of handling errors will come later
            pub fn tokenize(&mut self) -> Result<Vec<Token>, ()> {
                let mut tok: Vec<Token> = vec![];
                tok.push(Token::new(
                    Span {
                        start: self.current_pos,
                        end: self.current_pos,
                        path: self.path,
                    },
                    TokenKind::SoI,
                ));
                loop {
                    let ch = self.source.chars().nth(usize::from(self.current_pos));
                    match ch {
                        Some(c) => {
                            let state = LexerState::new(
                                self.current_pos,
                                self.source
                                    .get(usize::from(self.current_pos)..self.source.len())
                                    .unwrap(),
                                self.path,
                            );
                            let mut counter = 0;
                            for f in &self.sys {
                                let mut current_state = state.clone();
                                match f(c, &mut current_state) {
                                    Some(f) => {
                                        if $ignore {
                                            match f.kind() {
                                                TokenKind::WhiteSpace => {}
                                                TokenKind::CarriageReturn => {}
                                                TokenKind::NewLine => {}
                                                TokenKind::Tabulation => {}
                                                _ => tok.push(f),
                                            }
                                        } else {
                                            tok.push(f);
                                        }
                                        self.current_pos = current_state.current_pos;
                                        break;
                                    }
                                    None => {
                                        counter += 1;
                                        continue;
                                    }
                                }
                            }
                            if counter >= self.sys.len() {
                                return Err(());
                            }
                        }
                        None => break,
                    }
                }
                tok.push(Token::new(
                    Span {
                        start: self.current_pos,
                        end: self.current_pos,
                        path: self.path,
                    },
                    TokenKind::EoI,
                ));
                return Ok(tok);
            }
        }
        fn default_number(c: char, state: &mut LexerState) -> Option<Token> {
            if c.is_numeric() {
                let start = state.current_pos;
                let mut n = String::new();
                n.push(c);
                state.next();
                loop {
                    if let Some(c) = state.peek() {
                        if c.is_numeric() {
                            n.push(*c);
                            state.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if let Some(c) = state.peek() {
                    if *c == '.' {
                        n.push(*c);
                        state.next();
                        loop {
                            if let Some(c) = state.peek() {
                                if c.is_numeric() {
                                    n.push(*c);
                                    state.next();
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                Some(Token::new(
                    Span {
                        start,
                        end: state.current_pos,
                        path: state.path,
                    },
                    TokenKind::Literal(Literal::Float(n.parse::<f64>().unwrap())),
                ))
            } else {
                None
            }
        }
        fn default_whitespace(c: char, state: &mut LexerState) -> Option<Token> {
            let start = state.current_pos;
            let tok = match c {
                ' ' => TokenKind::WhiteSpace,
                '\t' => TokenKind::Tabulation,
                '\n' => TokenKind::NewLine,
                '\r' => TokenKind::CarriageReturn,
                _ => return None,
            };
            state.next();
            return Some(Token::new(
                Span {
                    start,
                    end: state.current_pos,
                    path: state.path,
                },
                tok,
            ))
        }
    };
    () => {
        lexer_builder!(ignore_space: true);
    }
}
/// To be done
#[macro_export]
macro_rules! symbols {
    ($($sym:literal => $variant:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct Token {
            span: Span,
            kind: TokenKind,
        }

        impl Spanned for Token {
            #[inline(always)]
            fn span(&self) -> Span {
                self.span
            }
        }

        impl Token {
            pub fn new(span: Span, kind: TokenKind) -> Self {
                Self { span, kind }
            }
            #[inline(always)]
            pub fn kind(&self) -> TokenKind {
                self.kind
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Literal {
            ///Default int literal, may change in the parser based on the type of the variable
            Int(i64),

            ///Default float literal, may change in the parser based on the type of the variable
            Float(f64),

            Bool(bool),
            //At this point, types don't exist in the parser, it's just Identifier
            Identifier(Intern<String>),

            StringLiteral(Intern<String>),
        }

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum TokenKind {
            /// A literal see [Literal] for more information
            Literal(Literal),

            /// A keyword
            Keyword(Intern<String>),
            $(
                $variant,
            )*
            WhiteSpace,
            NewLine,
            Tabulation,
            CarriageReturn,
            EoI,
            SoI
        }
        //TODO: add support for multi-char symbols
        fn default_symbol(c: char, state: &mut LexerState) -> Option<Token> {
            let start = state.current_pos;
            let tok = match c {
                $(
                    $sym => TokenKind::$variant,
                )*
                _ => return None,
            };
            state.next();
            Some(Token::new(
                Span {
                    start,
                    end: state.current_pos,
                    path: state.path,
                },
                tok,
            ))
        }
    };
    () => {
        symbols!{
            '+' => Plus,
            '-' => Minus,
            '*' => Asterisk,
            '/' => Slash,
            '%' => Percent,
            '=' => Equal,
            '<' => LessThan,
            '>' => GreaterThan,
            '!' => Exclamation,
            '&' => Ampersand,
            '|' => Pipe,
            '^' => Caret,
            '~' => Tilde,
            '(' => LeftParen,
            ')' => RightParen,
            '[' => LeftBracket,
            ']' => RightBracket,
            '{' => LeftBrace,
            '}' => RightBrace,
            '.' => Dot,
            ',' => Comma,
            ';' => Semicolon,
            ':' => Colon,
            '?' => Question,
            '#' => Hash,
            '$' => Dollar,
            '@' => At,
            '\\' => Backslash,
            '\'' => SingleQuote,
            '"' => DoubleQuote,
            '`' => Backtick
        }
    };
}
/// To be done
#[macro_export]
macro_rules! keywords {
    ($($x:literal),* $(,)?) => {
        use std::collections::HashMap;
        fn default_keyword(c: char, state: &mut LexerState) -> Option<Token> {
            let start = state.current_pos;
            let mut s = String::new();
            if c.is_alphabetic() || c == '_' {
                s.push(c);
                state.next();
                let keywords: HashMap<Intern<String>, TokenKind> = map! {
                    $(
                        Intern::new(String::from($x)) => TokenKind::Keyword(Intern::new(String::from($x))),
                    )*
                };
                loop {
                    if let Some(c) = state.peek() {
                        if c.is_alphabetic() || *c == '_' {
                            s.push(*c);
                            state.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if let Some(k) = keywords.get(&Intern::new(s)) {
                    Some(Token::new(Span {
                        start,
                        end: state.current_pos,
                        path: state.path
                    }, *k))
                } else {
                    return None;
                }
            } else {
                None
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros() {
        use crate::prelude::*;
        tmp_lexer_builder!(
            DefaultSystem {
                number: true,
                symbol: true,
                keyword: true,
                whitespace: {
                    allow_them: true,
                    use_system: true,
                },
            },
            Symbols {
                '.' => DOT,
            },
            Keyword { },
            Number {
                trailing: true,
                float: true,
                u_int: true,
                int: true
            }
        );
    }
}
