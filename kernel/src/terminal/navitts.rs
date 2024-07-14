// WIP i realized i need an allocator first

// kernel implementation of Navi Terminal Textual Escape Sequences
// it appears to be mostly in the format \[attributes | slice|]
// each attribute has a value \[attribute: value |slice|] a value can be a\an:
/*
    byte: it is 8 bits, default as base 10 and has a hex form where it starts with 0x and then digits which is of base 16 or it starts with 0b and it is digits are base
    tuple: has a var number of items sep by , stars with ( and ends with  )
    calle: a call expression to one of the built-in functions, starts with a name then a tuple with items supplied to function

    there is no other values, for now there is no binary expressions or whatever
*/

use core::str::Chars;

enum Token {
    TupleStart,
    TupleEnd,
    Char(char),
    SliceStart,
    SliceEnd,
    Comma,
    Byte(u8),
    HexByte(u8),
}

pub enum Type<'a> {
    Tuple(&'a [Type<'a>]),
    Byte,
}

pub enum Value<'a> {
    Byte(u8),
    Tuple(&'a [Value<'a>]),
    Calle(&'a str, &'a [Value<'a>]),
}

type Attribute = (&'static str, Type<'static>);
// output
pub enum NaviTTES<'a> {
    NaviES(&'a [Attribute], &'a [NaviTTES<'a>]),
    Slice(&'a str),
}

const ATTRIBUTES: &[Attribute] = &[("fg", Type::Tuple(&[Type::Byte, Type::Byte, Type::Byte]))];

impl<'a> NaviTTES<'a> {
    pub fn parse_str(str: &'a str) -> &[Self] {
        // first we look for an \[ in str
        let mut chars = str.chars();

        for (index, c) in chars.clone().enumerate() {
            if c == '\\' {
                if str.len() > index + 2 {
                    match (chars).nth(index + 1).as_ref().unwrap() {
                        '[' => {
                            // lexing
                            chars.advance_by(index + 3).unwrap();
                            let _tokens = Self::lex(chars.clone());
                        }
                        _ => continue,
                    }
                }
            }
        }

        todo!()
    }

    fn lex(chars: Chars) -> &[Token] {
        // terminates at ]
        for (_index, c) in chars.clone().enumerate() {
            if c == ']' {
                break;
            }
        }

        todo!()
    }
}
