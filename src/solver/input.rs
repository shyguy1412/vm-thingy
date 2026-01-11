use super::state::Location;
use std::io::Read;

pub enum ParseError {
    NotALocation,
    InvalidToken(char),
    UnexpectedToken(String, String),
    IOError,
}

enum Token {
    Item,
    LocationName,
    LocationDescription,
    ActionPrompt,
}

pub enum Block {
    Location(Location),
}

pub struct Input {
    inner: Box<dyn Read>,
    current: char,
    lookahead: char,
}

impl<T: Read + 'static> From<T> for Input {
    fn from(mut value: T) -> Self {
        //prime the input
        let buf = &mut [0];
        let _ = value.read_exact(buf);

        Self {
            inner: Box::new(value),
            lookahead: buf[0] as char,
            current: 0x00 as char,
        }
    }
}

impl Iterator for Input {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_block()
            .inspect_err(|e| match e {
                ParseError::NotALocation => println!("NotALocation"),
                ParseError::InvalidToken(token) => println!("InvalidToken: `{token}`"),
                ParseError::UnexpectedToken(expected, got) => {
                    println!("UnexpectedToken: \n`{got}` expected \n`{expected}`")
                }
                ParseError::IOError => println!("IOError"),
            })
            .ok()
    }
}

impl Input {
    pub fn init(&mut self) {
        while self.lookahead != '=' {
            let _ = self.next_char();
        }
    }

    pub fn parse_block(&mut self) -> Result<Block, ParseError> {
        match self.lookahead {
            '=' => self.parse_location(),
            char => Err(ParseError::InvalidToken(char)),
        }
    }

    pub fn parse_location(&mut self) -> Result<Block, ParseError> {
        let name = self.consume(Token::LocationName)?;
        let description = self.consume(Token::LocationDescription)?;

        Ok(Block::Location(Location::new(name, description)))
    }

    fn consume(&mut self, token: Token) -> Result<String, ParseError> {
        match token {
            Token::LocationName => self.consume_location_name(),
            Token::LocationDescription => self.consume_location_description(),
            Token::Item => todo!(),
            Token::ActionPrompt => todo!(),
        }
    }

    fn consume_location_name(&mut self) -> Result<String, ParseError> {
        self.consume_str("== ")?;
        let mut name: String = String::new();
        while self.lookahead != '=' {
            name.push(self.next_char()?);
        }
        name.pop(); // pop off trailing space
        self.consume_str("==\n")?;

        Ok(name)
    }

    fn consume_location_description(&mut self) -> Result<String, ParseError> {
        let mut name: String = String::new();
        while self.lookahead != '\n' {
            name.push(self.next_char()?);
        }
        self.consume_str("\n\n")?;

        Ok(name)
    }

    fn consume_str(&mut self, str: &str) -> Result<String, ParseError> {
        let mut result = String::new();
        for _ in 0..str.len() {
            result.push(self.next_char()?);
        }
        match result == str {
            true => Ok(result),
            false => Err(ParseError::UnexpectedToken(str.to_string(), result)),
        }
    }

    fn next_char(&mut self) -> Result<char, ParseError> {
        self.current = self.lookahead;

        let mut buf = [0];
        let Ok(_) = self.inner.read(&mut buf) else {
            return Err(ParseError::IOError);
        };
        self.lookahead = buf[0] as char;

        Ok(self.current)
    }

    fn read_line(&mut self) -> Result<String, ParseError> {
        let mut line: String = String::new();
        while self.current != '\n' {
            line.push(self.next_char()?);
        }
        self.next_char()?; //consume newline
        Ok(line)
    }
}
