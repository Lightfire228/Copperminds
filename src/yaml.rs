

pub fn scan(text: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(text);

    while scanner.has_next() {
        scanner.scan_next();
    }

    scanner.push_eof();

    scanner.tokens
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Text does not imply "string" data, but rather anything that didn't match another token
    Text, Pound,
    
    Digits, Dot,

    Whitespace, Newline,

    Quote, DoubleQuote, Colon, Star, Ampersand,
    Backslash, Bang, Comma, Minus,

    LeftBracket, RightBracket,
    LeftCurly,   RightCurly,
    LeftAngle,   RightAngle,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub type_:  TokenType,
    pub lexeme: String,
    pub line:   usize,
    pub col:    usize,
}


#[derive(Debug)]
struct Scanner {
    tokens:             Vec<Token>,
    
    chars:              Vec<char>,
    cursor:             Cursor,
    text:               Vec<char>,
    text_cursor:        Cursor,
}

#[derive(Copy, Clone, Debug)]
struct Cursor {
    pub index: usize,
    pub line:  usize,
    pub col:   usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            index: 0,
            line:  1,
            col:   1,
        }
    }
}

impl Scanner {
    pub fn new(text: &str) -> Self {

        Self {
            tokens: vec![],
            chars:  text.chars().collect(),

            cursor:       Cursor::new(),
            text:         vec![],
            text_cursor:  Cursor::new(),
        }
    }

    pub fn has_next(&self) -> bool {
        self.cursor.index < self.chars.len()
    }

    pub fn scan_next(&mut self) {
        
        let ch = self.advance();
        
        let     prv_index    = self.cursor.index;
        let mut matched_text = false;

        match ch {
            '#'         => self.push_token_ch  (TokenType::Pound),
            '.'         => self.push_token_ch  (TokenType::Dot),
            '\''        => self.push_token_ch  (TokenType::Quote),
            '"'         => self.push_token_ch  (TokenType::DoubleQuote),
            ':'         => self.push_token_ch  (TokenType::Colon),
            '*'         => self.push_token_ch  (TokenType::Star),
            '&'         => self.push_token_ch  (TokenType::Ampersand),
            '\\'        => self.push_token_ch  (TokenType::Backslash),
            '!'         => self.push_token_ch  (TokenType::Bang),
            ','         => self.push_token_ch  (TokenType::Comma),
            '-'         => self.push_token_ch  (TokenType::Minus),
            '{'         => self.push_token_ch  (TokenType::LeftCurly),
            '}'         => self.push_token_ch  (TokenType::RightCurly),
            '['         => self.push_token_ch  (TokenType::LeftBracket),
            ']'         => self.push_token_ch  (TokenType::RightBracket),
            '<'         => self.push_token_ch  (TokenType::LeftAngle),
            '>'         => self.push_token_ch  (TokenType::RightAngle),
            ' '  | '\t' => self.push_whitespace(),
            '\n' | '\r' => self.push_newline   (),
            '0'..='9'   => self.push_digits    (),
            _           => {
                self.text.push(ch);
                matched_text = true;
            }
        }


        if !matched_text && !self.text.is_empty() {
            let prv_token = self.tokens.pop().unwrap();

            let lexeme = self.lexeme(self.text_cursor.index, prv_index -1);

            self.push_token_lexeme(lexeme, TokenType::Text);
            self.tokens.push(prv_token);
        }

        if !matched_text {
            self.text_cursor = self.cursor;
            self.text.clear();
        }
    }

    pub fn push_eof(&mut self) {

        if !self.text.is_empty() {
            let lexeme = self.lexeme(self.text_cursor.index, self.cursor.index);
            self.push_token_lexeme(lexeme, TokenType::Text);
        }

        self.tokens.push(Token {
            type_:  TokenType::EOF,
            lexeme: String::new(),
            line:   self.cursor.line,
            col:    self.cursor.col,
        });
    }

    fn push_token_lexeme(&mut self, lexeme: String, type_: TokenType) {
        self.tokens.push(Token {
            type_,
            lexeme,
            line:   self.cursor.line,
            col:    self.cursor.col,
        });
    }

    fn push_token_ch(&mut self, type_: TokenType) {
        self.tokens.push(Token {
            type_,
            lexeme: String::from(self.ch()),
            line:   self.cursor.line,
            col:    self.cursor.col,
        });
    }

    fn push_newline(&mut self) {


        let start = self.cursor.index -1;
        
        if self.ch() == '\r' && self.next() == '\n' {
            self.cursor.index += 1;
        }

        self.tokens.push(Token {
            type_:  TokenType::Newline,
            lexeme: self.lexeme(start, self.cursor.index),
            line:   self.cursor.line,
            col:    self.cursor.col,
        });
        
        self.cursor = Cursor {
            index: self.cursor.index,
            line:  self.cursor.line +1,
            col:   1,
        };
    }

    fn ch(&self) -> char {
        self.chars[self.cursor.index -1]
    }

    fn next(&self) -> char {
        self.chars[self.cursor.index]
    }

    fn advance(&mut self) -> char {
        self.cursor = Cursor {
            index: self.cursor.index + 1,
            col:   self.cursor.col   + 1,
            line:  self.cursor.line,
        };
        self.ch()
    }

    fn lexeme(&self, start: usize, end: usize) -> String {
        self.chars[start..end]
            .iter             ()
            .cloned           ()
            .collect::<String>()
    }



    fn push_digits(&mut self) {
        let start = self.cursor.index -1;

        while '0' <= self.next() && self.next() <= '9' {
            self.advance();
        }

        let lexeme = self.lexeme(start, self.cursor.index);
        self.push_token_lexeme(lexeme, TokenType::Digits);
    }

    
    fn push_whitespace(&mut self) {
        let start = self.cursor.index -1;
        
        // TODO: vertical tabs?
        while self.next() == ' ' || self.next() == '\t' {
            self.advance();
        }

        let lexeme = self.lexeme(start, self.cursor.index);
        self.push_token_lexeme(lexeme, TokenType::Whitespace);
    }
}



#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn base() {

        fn mk_token(lex: &str, type_: TokenType) -> Token {
            Token {
                type_,
                lexeme: String::from(lex),
                line:   0,
                col:    0,
            }
        }

        let strs = vec![
            "test",
            "",
            "      \ttest_whitespace",
            "123",
            "#*!&:.,-'\"\\[]<>{}",
            "#test*test!test&test:test.test,test-test'test\"test\\test[test]test<test>test{test}test",
        ];
        
        fn test_token() -> Token { mk_token("test", TokenType::Text)    }
        fn ln_token  () -> Token { mk_token("\n",   TokenType::Newline) }


        let mut tokes = vec![
            test_token(), ln_token(),
            ln_token(),
            mk_token("      \t", TokenType::Whitespace), mk_token("test_whitespace", TokenType::Text), ln_token(),
            mk_token("123",      TokenType::Digits),     ln_token(),
        ];

        fn symbols() -> Vec<Token> { vec![
            mk_token("#",  TokenType::Pound),
            mk_token("*",  TokenType::Star),
            mk_token("!",  TokenType::Bang),
            mk_token("&",  TokenType::Ampersand),
            mk_token(":",  TokenType::Colon),
            mk_token(".",  TokenType::Dot),
            mk_token(",",  TokenType::Comma),
            mk_token("-",  TokenType::Minus),
            mk_token("'",  TokenType::Quote),
            mk_token("\"", TokenType::DoubleQuote),
            mk_token("\\", TokenType::Backslash),
            mk_token("[",  TokenType::LeftBracket),
            mk_token("]",  TokenType::RightBracket),
            mk_token("<",  TokenType::LeftAngle),
            mk_token(">",  TokenType::RightAngle),
            mk_token("{",  TokenType::LeftCurly),
            mk_token("}",  TokenType::RightCurly),
        ]}

        tokes.extend(symbols());
        tokes.push  (ln_token());

        tokes.extend(symbols().into_iter().flat_map(|f| vec![f, test_token()]));
        tokes.push  (mk_token("", TokenType::EOF));

        let text = strs.join("\n");

        let tokens = scan(&text);

        for (t1, t2) in tokens.iter().zip(tokes.iter()) {
            println!("{:?} - {:?}", t1.type_,  t2.type_);
            println!("{:?} - {:?}", t1.lexeme, t2.lexeme);
            println!("");

            assert!(t1.type_  == t2.type_,  "Token Types don't match");
            assert!(t1.lexeme == t2.lexeme, "Lexemes don't match");
        }

    }

}

