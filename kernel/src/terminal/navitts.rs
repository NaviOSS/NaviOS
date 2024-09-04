// EXTERMELY BAD CODE WARNING (AND NAMING)
// it is better then the bimble programming language compiler src at least
// this was made in a rush
// TODO
// - make better code, also current code is lacking alot like escaping | and inlining more ess' in slices
// - figure out better names for all this bs
// - comments

// kernel implementation of Navi Terminal Textual Escape Sequences
// it appears to be mostly in the format \[attributes ||slice||]
// each attribute has a value \[attribute: value ||slice||] a value can be a\an:
/*
    byte: it is 8 bits, default as base 10 and has a hex form where it starts with 0x and then digits which is of base 16 or it starts with 0b and it is digits are base
    tuple: has a var number of items sep by , stars with ( and ends with  )
    calle: a call expression to one of the built-in functions, starts with a name then a tuple with items supplied to function

    there is no other values, for now there is no binary expressions or whatever
*/

use core::{iter::Peekable, str::Chars};

use alloc::{boxed::Box, string::String, vec::Vec};

#[derive(Debug, PartialEq, Clone)]
enum Token {
    TupleStart,
    TupleEnd,
    Ident(String),
    Slice(String),
    Comma,
    Colon,
    Byte(u8),
}

#[derive(Debug, Clone)]
pub enum Value {
    Byte(u8),
    Tuple(Vec<Value>),
}

type Attribute = (String, Value);

#[derive(Debug, Clone)]
pub enum NaviTTES<'a> {
    NaviES(Vec<Attribute>, Box<NaviTTES<'a>>),
    NaviESS(Vec<NaviTTES<'a>>),
    Slice(&'a str),
    OwnedSlice(String),
}

#[derive(Debug, Clone)]
pub struct Attributes {
    pub fg: (u8, u8, u8),
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            fg: (255, 255, 255),
        }
    }
}

impl Attributes {
    pub fn from_list(attr_list: &[Attribute], default: Attributes) -> Self {
        let mut attrs = default;

        for (key, value) in attr_list {
            match (key.as_str(), value) {
                ("fg", Value::Tuple(ref vals)) => {
                    if let [Value::Byte(r), Value::Byte(g), Value::Byte(b)] = vals[..] {
                        attrs.fg = (r, g, b);
                    }
                }

                _ => {}
            }
        }

        attrs
    }
}
impl<'a> NaviTTES<'a> {
    pub fn parse_str(str: &'a str) -> Self {
        let mut result = NaviTTES::Slice(str);
        let mut chars = str.chars().peekable();

        while let Some(char) = chars.next() {
            match char {
                '\\' => match chars.peek() {
                    Some('[') => {
                        chars.next();
                        let mut ttes = match result.clone() {
                            NaviTTES::NaviESS(ttes) => ttes,
                            NaviTTES::Slice(_) => Vec::from([result.clone()]),
                            _ => todo!(),
                        };

                        let tokens = Self::lex_chars(&mut chars);
                        let Ok(val) = Parser::parse_tokens(tokens) else {
                            continue;
                        };

                        ttes.push(val);
                        ttes.push(NaviTTES::OwnedSlice(String::new()));

                        result = NaviTTES::NaviESS(ttes);
                        continue;
                    }
                    _ => (),
                },
                _ => (),
            }

            let mut last = if let NaviTTES::NaviESS(ref mut results) = result {
                results.last_mut()
            } else {
                None
            };

            if let Some(ref mut result) = last {
                match result {
                    NaviTTES::OwnedSlice(ref mut s) => s.push(char),
                    _ => unreachable!(),
                }
            }

            drop(last)
        }

        // remove the first item if we got more then one ess because it is useless
        // it was added as a no allocation strategy when there is no escapes
        if let NaviTTES::NaviESS(ref mut results) = result {
            if results.len() > 1 {
                results.remove(0);
            }
        }

        return result;
    }

    fn lex_chars(chars: &mut Peekable<Chars>) -> Vec<Token> {
        let mut results = Vec::new();
        while let Some(char) = chars.next() {
            let token = match char {
                ']' => {
                    break;
                }
                ' ' | '\n' | '\t' => continue,

                ':' => Token::Colon,
                ',' => Token::Comma,
                '(' => Token::TupleStart,
                ')' => Token::TupleEnd,

                '0'..='9' => Token::Byte(Self::lex_number(chars, char)),
                'a'..='z' | 'A'..='Z' => Token::Ident(Self::lex_ident(chars, char)),
                '|' => {
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        Token::Slice(Self::lex_slice(chars))
                    } else {
                        continue;
                    }
                }
                _ => return Vec::new(),
            };

            results.push(token)
        }

        results
    }

    fn lex_number(chars: &mut Peekable<Chars>, first: char) -> u8 {
        let mut results = String::new();
        results.push(first);

        while let Some(char) = chars.peek() {
            if ('0'..'9').contains(char) {
                results.push(char.clone());
                chars.next();
            } else {
                break;
            }
        }

        results.parse().unwrap()
    }

    fn lex_ident(chars: &mut Peekable<Chars>, first: char) -> String {
        let mut results = String::new();
        results.push(first);

        while let Some(char) = chars.peek() {
            if ('a'..'z').contains(&char) | ('A'..'Z').contains(&char) {
                results.push(char.clone());
                chars.next();
            } else {
                break;
            }
        }

        results
    }

    fn lex_slice(chars: &mut Peekable<Chars>) -> String {
        let mut results = String::new();

        while let Some(char) = chars.next() {
            if char == '|' {
                if chars.peek() == Some(&'|') {
                    break;
                }
            } // for now you cannot use the char '||' in your slice TODO add an escape for that

            results.push(char)
        }

        results
    }
}

#[derive(Clone)]
struct Parser<'a> {
    tokens: &'a [Token],
    current_token: usize,
}

impl<'a, 'r> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            current_token: 0,
        }
    }

    fn current(&self) -> Token {
        self.tokens[self.current_token].clone()
    }

    fn eat(&mut self) -> Token {
        let prev = self.current();
        self.current_token += 1;
        prev
    }

    fn expect(&mut self, expect: Token) -> Result<Token, ()> {
        let eat = self.eat();
        if eat == expect {
            Ok(eat)
        } else {
            Err(())
        }
    }

    pub(super) fn parse_tokens(tokens: Vec<Token>) -> Result<NaviTTES<'r>, ()> {
        let mut parser = Parser::new(&tokens);
        let mut ttes = Vec::new();

        while parser.current_token < parser.tokens.len() {
            match parser.current() {
                Token::Ident(_) => {
                    let attributes = parser.parse_attributes()?;
                    let Token::Slice(slice) = parser.eat() else {
                        return Err(());
                    };

                    let slice_ttess = { NaviTTES::OwnedSlice(slice) };

                    ttes.push(NaviTTES::NaviES(attributes, Box::new(slice_ttess)))
                }
                Token::Slice(s) => ttes.push(NaviTTES::OwnedSlice(s)),
                _ => return Err(()),
            }
        }

        Ok(NaviTTES::NaviESS(ttes))
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ()> {
        let mut results = Vec::new();

        results.push(self.parse_attribute()?);

        while self.current() == Token::Comma {
            self.eat();
            let attr = self.parse_attribute()?;
            results.push(attr)
        }

        Ok(results)
    }

    fn parse_attribute(&mut self) -> Result<Attribute, ()> {
        let Token::Ident(id) = self.eat() else {
            return Err(());
        };

        self.expect(Token::Colon)?;
        let val = self.parse_value()?;

        // for (attr_name, _) in ATTRIBUTES {
        //     if *attr_name == id.as_str() {
        //         return Ok((attr_name, val));
        //     }
        // }
        return Ok((id, val));
    }

    fn parse_value(&mut self) -> Result<Value, ()> {
        match self.eat() {
            Token::Byte(b) => Ok(Value::Byte(b)),
            Token::TupleStart => {
                let mut items = Vec::new();

                items.push(self.parse_value()?);
                while self.current() == Token::Comma {
                    self.eat();
                    items.push(self.parse_value()?);
                }

                self.expect(Token::TupleEnd)?;
                Ok(Value::Tuple(items))
            }
            a => todo!("value {:#?}", a),
        }
    }
}
