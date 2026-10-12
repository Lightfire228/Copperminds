use core::fmt;
use std::vec::IntoIter;

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

pub fn parse(tokens: Vec<Token>) -> Result<Vec<CommandExpr>, ParseError> {
    let mut tokens = tokens.into_iter();

    let mut commands = vec![];

    while let Some(operator) = tokens.next() {
        commands.push(parse_operator(operator, &mut tokens)?);
    }

    Ok(commands)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind:   TokenKind,
    pub lexeme: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Colon,
    At,
    Plus, Minus,
    // Star, Slash,

    Text,
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

    fn tokenize_error(&self, args: fmt::Arguments) -> ! {
        panic!("{}", args)
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
    pub value:    String,
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

fn parse_operator(op: Token, tokens: &mut IntoIter<Token>) -> Result<CommandExpr, ParseError> {

    match op.kind {
        TokenKind::Colon => parse_set_prop(op, tokens),

        _ => Err(ParseError {
            kind:  ParseErrorKind::UnknownOperator,
            token: op,
        })?,
    }
}

fn parse_set_prop(op: Token, tokens: &mut IntoIter<Token>) -> Result<CommandExpr, ParseError> {


    type Ek = ParseErrorKind;
    let err = |kind| ParseError {
        kind,
        token: (&op).to_owned()
    };

    let target = tokens.next().ok_or_else(|| err(Ek::MissingTarget))?;

    Ok(match target.lexeme.as_str() {
        "d" => CommandExpr::Delete,
        "i" => CommandExpr::SetInfo,
        "a" => parse_set_action(target, tokens)?,
        "s" => parse_set_status(target, tokens)?,
        _   => Err(err(Ek::UnknownTarget))?
    })
}

fn parse_set_action(target: Token, tokens: &mut IntoIter<Token>) -> Result<CommandExpr, ParseError> {

    type Ek = ParseErrorKind;
    let err = |kind| ParseError {
        kind,
        token: (&target).to_owned()
    };

    let value = tokens.next().ok_or_else(|| err(Ek::MissingValue ))?;

    let value: FmAction = value.lexeme.as_str().try_into().map_err(|_| err(Ek::UnknownValue))?;

    Ok(CommandExpr::SetAction(value))
}

fn parse_set_status(target: Token, tokens: &mut IntoIter<Token>) -> Result<CommandExpr, ParseError> {

    type Ek = ParseErrorKind;
    let err = |kind| ParseError {
        kind,
        token: (&target).to_owned()
    };

    let value = tokens.next().ok_or_else(|| err(Ek::MissingValue ))?;

    let value: FmStatus = value.lexeme.as_str().try_into().map_err(|_| err(Ek::UnknownValue))?;

    Ok(CommandExpr::SetStatus(value))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let str = ":a todo @s archive +b \"asdf\" -d 'qwerty\\' uiop'";

        let tokens = tokenize(str).unwrap();

        macro_rules! token {
            ($kind:expr, $lexeme:expr) => {
                Token {
                    kind:   $kind,
                    lexeme: $lexeme.to_string(),
                }
            };
        }

        type Tk = TokenKind;
        assert_eq!(tokens, vec![
            token!(Tk::Colon, ":"),
            token!(Tk::Text,  "a"),
            token!(Tk::Text,  "todo"),

            token!(Tk::At,    "@"),
            token!(Tk::Text,  "s"),
            token!(Tk::Text,  "archive"),

            token!(Tk::Plus,  "+"),
            token!(Tk::Text,  "b"),
            token!(Tk::Text,  "asdf"),

            token!(Tk::Minus, "-"),
            token!(Tk::Text,  "d"),
            token!(Tk::Text,  "qwerty\' uiop"),
        ])

    }

    #[test]
    fn test_parser() {
        let str = ":a todo :s archived :d :i";

        let tokens   = tokenize(str)   .unwrap();
        let commands = parse   (tokens).unwrap();

        type Tk = TokenKind;
        assert_eq!(commands, vec![
            CommandExpr::SetAction(FmAction::Todo),
            CommandExpr::SetStatus(FmStatus::Archived),
            CommandExpr::Delete,
            CommandExpr::SetInfo,
        ])

    }

}
