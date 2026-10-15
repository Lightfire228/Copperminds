use core::fmt;
use std::{iter::Peekable, vec::IntoIter};

use enum_iterator::{Sequence, all};

use crate::vault::fm::{FmAction, FmStatus};

pub fn tokenize(str: &str) -> TResult<Vec<Token>> {

    let chars: Vec<_> = str.chars().collect();

    let mut tokenizer = Tokenizer {
        index:  0,
        tokens: vec![],
    };

    while tokenizer.parse_expression(&chars)? {}

    Ok(tokenizer.tokens)

}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<CommandGroup>, ParseError> {
    let mut parser = Parser {
        tokens: tokens.into_iter().peekable(),
    };
    let mut commands = vec![];

    while let Some(operator) = parser.tokens.next() {
        commands.push(parser.parse_operator(operator)?);
    }

    Ok(commands)
}

pub fn parse_groups(groups: Vec<CommandGroup>) -> Result<Vec<CommandExpr>, ParseError> {
    groups
        .into_iter()
        .map      (|g| g.parse_group())
        .collect()
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind:   TokenKind,
    pub lexeme: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence)]
pub enum TokenKind {
    Colon,
    At,
    Plus, Minus,
    // Star, Slash,

    Text,
}

impl TokenKind {
    pub const fn is_operator(&self) -> bool {
        matches!(self,
              TokenKind::Colon
            | TokenKind::At
            | TokenKind::Plus
            | TokenKind::Minus
        )
    }
}


#[derive(Debug, PartialEq, Eq)]
pub enum TokenizerError {
    UnknownChar       (char),
    UnknownEscape     (char),
    UnterminatedString,

}

type TResult<T> = Result<T, TokenizerError>;

#[derive(Debug)]
struct Tokenizer {
    index:  usize,
    tokens: Vec<Token>,
}


impl Tokenizer {

    fn parse_expression(&mut self, chars: &[char]) -> TResult<bool> {
        self.skip_whitespace(chars);

        type Tk = TokenKind;

        fn s(str: &str) -> String {
            str.to_string()
        }

        if !self.has_next(chars) {
            return Ok(false);
        }

        let char = chars[self.index];

        let mut op = |token: Token| {
            self.index += token.lexeme.len();
            self.tokens.push(token);
        };



        match char {
            ':'  => op(Token { kind: TokenKind::Colon, lexeme: s(":") }),
            '@'  => op(Token { kind: TokenKind::At,    lexeme: s("@") }),
            '+'  => op(Token { kind: TokenKind::Plus,  lexeme: s("+") }),
            '-'  => op(Token { kind: TokenKind::Minus, lexeme: s("-") }),
            // '*' => Token { kind: TokenKind::Star,  lexeme: s("*") },
            // '/' => Token { kind: TokenKind::Slash, lexeme: s("/") },

            x @ ('\'' | '\"') => self.parse_string(chars, x)?,

            'a'..='z' |
            'A'..='Z' |
            '0'..='9' |
            '_'       => self.parse_ident(chars)?,

            x => Err(TokenizerError::UnknownChar(x))?
        };

        Ok(true)
    }

    fn skip_whitespace(&mut self, chars: &[char])  {

        while self.has_next(chars) && matches!(chars[self.index], ' ' | '\t' | '\n') {
            self.index += 1;
        };
    }


    fn parse_string(&mut self, chars: &[char], closing: char) -> TResult<()> {

        println!("start: {}", self.index);

        // consume the opening '
        self.index += 1;

        let mut str = String::new();

        macro_rules! ch {
            () => { chars[self.index] };
        }

        if !self.has_next(chars) {
            Err(TokenizerError::UnterminatedString)?
        }

        while ch!() != closing {

            if ch!() == '\\' {
                self.index += 1;

                str.push(self.parse_escape(chars)?);
                self.index += 1;

                continue;
            }

            str.push(ch!());
            self.index += 1;

            if !self.has_next(chars) {
                Err(TokenizerError::UnterminatedString)?
            }

        }

        // consume the closing '
        self.index += 1;

        self.tokens.push(Token {
            kind:   TokenKind::Text,
            lexeme: str,
        });

        Ok(())
    }

    fn parse_escape(&mut self, chars: &[char]) -> TResult<char> {
        if !self.has_next(chars) {
            Err(TokenizerError::UnterminatedString)?
        }

        Ok(match chars[self.index] {
            ch @ (
                '\\' |
                '\'' |
                '\"' |
                '\n' |
                '\t'
            ) => ch,

            x => Err(TokenizerError::UnknownEscape(x))?
        })
    }

    fn parse_ident(&mut self, chars: &[char]) -> TResult<()> {

        let mut str = String::new();

        str.push(chars[self.index]);
        self.index += 1;

        while self.has_next(chars) && matches!(
            chars[self.index],
            'a'..='z' |
            'A'..='Z' |
            '0'..='9' |
            '_'
        ) {
            str.push(chars[self.index]);
            self.index += 1;

        }

        self.tokens.push(Token {
            kind:   TokenKind::Text,
            lexeme: str,
        });

        Ok(())
    }

    #[inline(always)]
    fn has_next(&self, chars: &[char]) -> bool {
        self.index < chars.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandExpr {
    SetAction(FmAction),
    SetStatus(FmStatus),
    SetInfo,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandGroup {
    pub operator: Token,
    pub target:   Token,
    pub value:    Option<Token>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind:  ParseErrorKind,
    pub token: Token,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnknownOperator,
    UnknownTarget,
    UnknownValue,
    MissingTarget,
    MissingValue,
}

struct Parser {
    tokens:    Peekable<IntoIter<Token>>,
}

macro_rules! parse_err {
    ($token:expr, $kind:ident) => {
        Err(ParseError {
            kind:  ParseErrorKind::$kind,
            token: $token,
        })
    };
}
impl Parser {

    fn parse_operator(&mut self, op: Token) -> Result<CommandGroup, ParseError> {

        let is_operator = matches!(op.kind,
              TokenKind::Colon
            | TokenKind::At
            | TokenKind::Minus
            | TokenKind::Plus
        );

        if !is_operator {
            return parse_err!(op, UnknownOperator);
        }

        let Some(target) = self.tokens.next() else {
            return parse_err!(op, MissingTarget);
        };

        let value = self.tokens.next_if(|x| x.kind == TokenKind::Text);

        Ok(CommandGroup {
            operator: op,
            target,
            value,
        })
    }

}

impl CommandGroup {
    fn parse_group(self) -> Result<CommandExpr, ParseError> {

        type Tk = TokenKind;
        Ok(match (self.operator.kind, self.target.lexeme.as_str()) {
            (Tk::Colon, "d") => CommandExpr::Delete,
            (Tk::Colon, "i") => CommandExpr::SetInfo,
            (Tk::Colon, "a") => self.parse_set_action()?,
            (Tk::Colon, "s") => self.parse_set_status()?,
            _ => parse_err!(self.operator, UnknownOperator)?
        })
    }
}

macro_rules! impl_parse {
    ($func:ident, $target:ident, $expr:ident) => {

        fn $func(self) -> Result<CommandExpr, ParseError> {

            let Some(value) = self.value else {
                return parse_err!(self.target, MissingValue);
            };


            let value: $target = value
                .lexeme
                .as_str  ()
                .try_into()
                .or_else (|_| $target::from_alias(&value.lexeme).ok_or(()))
                .map_err (|_| ParseError {
                    kind:  ParseErrorKind::UnknownValue,
                    token: value,
                })?
            ;

            Ok(CommandExpr::$expr(value))
        }
    };
}
impl CommandGroup {

    impl_parse!(parse_set_action, FmAction, SetAction);
    impl_parse!(parse_set_status, FmStatus, SetStatus);
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! token {
        ($kind:ident, $lexeme:expr) => {
            Token {
                kind:   TokenKind::$kind,
                lexeme: $lexeme.to_string(),
            }
        };

        ($kind:ident) => {
            Token {
                kind:   TokenKind::$kind,
                lexeme: TokenKind::$kind.get_op().to_string(),
            }
        };
    }

    impl TokenKind {
        fn get_op(&self) -> &str {
            match self {
                TokenKind::Colon => ":",
                TokenKind::At    => "@",
                TokenKind::Plus  => "+",
                TokenKind::Minus => "-",

                _ => unimplemented!("not an operator")
            }
        }
    }



    #[test]
    fn test_tokenizer() {
        let str = ":a todo @s archive +b \"asdf\" -d 'qwerty\\' uiop'";

        let tokens = tokenize(str).unwrap();

        assert_eq!(tokens, vec![
            token!(Colon),
            token!(Text,  "a"),
            token!(Text,  "todo"),

            token!(At),
            token!(Text,  "s"),
            token!(Text,  "archive"),

            token!(Plus),
            token!(Text,  "b"),
            token!(Text,  "asdf"),

            token!(Minus),
            token!(Text,  "d"),
            token!(Text,  "qwerty\' uiop"),
        ])

    }

    #[test]
    fn test_parser() {
        let str = ":a todo @s archive +b \"asdf\" -u -d 'qwerty\\' uiop'";

        let tokens   = tokenize(str)   .unwrap();
        let groups = parse   (tokens).unwrap();

        macro_rules! command {
            ($op:ident, $target:expr) => {
                CommandGroup {
                    operator: token!($op),
                    target:   token!(Text, $target),
                    value:    None,
                }
            };
            ($op:ident, $target:expr, $value:expr) => {
                CommandGroup {
                    operator: token!($op),
                    target:        token!(Text, $target),
                    value:    Some(token!(Text, $value)),
                }
            };
        }


        assert_eq!(groups, vec![
            command!(Colon, "a", "todo"),
            command!(At,    "s", "archive"),
            command!(Plus,  "b", "asdf"),
            command!(Minus, "u"),
            command!(Minus, "d", "qwerty\' uiop"),
        ])

    }

    #[test]
    fn test_command_parser() {
        let str = ":a todo :s archived :d :i";

        let tokens   = tokenize    (str)   .unwrap();
        let groups   = parse       (tokens).unwrap();
        let commands = parse_groups(groups).unwrap();

        assert_eq!(commands, vec![
            CommandExpr::SetAction(FmAction::Todo),
            CommandExpr::SetStatus(FmStatus::Archived),
            CommandExpr::Delete,
            CommandExpr::SetInfo,
        ])

    }
}
