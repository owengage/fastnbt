use std::borrow::Cow;

use nom::{IResult, combinator::{recognize, opt, map_res, map, cut}, sequence::{tuple, delimited, pair}, character::complete::{one_of, char, digit0, alphanumeric1, digit1}, branch::alt, bytes::complete::{tag, is_a, tag_no_case}, multi::many1, error::{ParseError, ErrorKind}};

pub fn parse_str<'a>(input: &'a str) -> IResult<&'a str, Cow<'a, str>> {
    alt((
        delimited(char('"'), parse_escaped('"'), char('"')),
        delimited(char('\''), parse_escaped('\''), char('\'')),
        map(parse_simple_string, |s: &str| Cow::from(s)),
    ))(input)
}

fn parse_escaped<'a, E: ParseError<&'a str>>(surround: char) -> impl FnMut(&'a str) -> IResult<&'a str, Cow<'a, str>, E> {
    move |input: &'a str| {
        let mut owned = String::new();
        let mut start = 0;
        let mut skip = false;
        let mut chars = input.chars();
        while let Some(c) = chars.next() {
            if skip {
                skip = false;
                owned.push(c);
                start = input.len() - chars.as_str().len();
            } else if c == '\\' {
                let len = input.len() - chars.as_str().len() - 1;
                owned.push_str(&input[start..len]);
                skip = true;
            } else if c == surround {
                let len = input.len() - chars.as_str().len() - surround.len_utf8();
                if !owned.is_empty() {
                    if len > start {
                        owned.push_str(&input[start..len]);
                    }
                    return Ok((&input[len..], Cow::from(owned)));
                } else {
                    return Ok((&input[len..], Cow::from(&input[..len])));
                }
            }
        }
        Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::MapRes)))
    }
}

fn parse_simple_string(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((
        alphanumeric1,
        is_a("_-.+"),
    ))))(input)
}

pub fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

pub fn parse_i8(input: &str) -> IResult<&str, i8> {
    map_res(|input| {
        let (input, num) = decimal(input)?;
        let (input, _) = alt((char('b'), char('B')))(input)?;
        Ok((input, num))
    }, |s: &str| s.parse())(input)
}

pub fn parse_i16(input: &str) -> IResult<&str, i16> {
    map_res(|input| {
        let (input, num) = decimal(input)?;
        let (input, _) = alt((char('s'), char('S')))(input)?;
        Ok((input, num))
    }, |s: &str| s.parse())(input)
}

pub fn parse_i32(input: &str) -> IResult<&str, i32> {
    map_res(decimal, |s: &str| s.parse())(input)
}

pub fn parse_i64(input: &str) -> IResult<&str, i64> {
    map_res(|input| {
        let (input, num) = decimal(input)?;
        let (input, _) = alt((char('l'), char('L')))(input)?;
        Ok((input, num))
    }, |s: &str| s.parse())(input)
}

pub fn parse_f32(input: &str) -> IResult<&str, f32> {
    map_res(|input| {
        let (input, num) = float(input)?;
        let (input, _) = alt((char('f'), char('F')))(input)?;
        Ok((input, num))
    }, |s: &str| s.parse())(input)
}

pub fn parse_f64(input: &str) -> IResult<&str, f64> {
    map_res(|input| {
        let (input, num) = float(input)?;
        let (input, _) = opt(alt((char('d'), char('D'))))(input)?;
        Ok((input, num))
    }, |s: &str| s.parse())(input)
}

fn float(input: &str) -> IResult<&str, &str> {
    alt((tag_no_case("inf"), tag_no_case("infinity"), tag_no_case("nan"),
        recognize(tuple((
            opt(alt((char('+'), char('-')))),
            alt((
                map(tuple((digit1, pair(char('.'), opt(digit1)))), |_| ()),
                map(tuple((char('.'), digit1)), |_| ())
            )),
            opt(tuple((
                alt((char('e'), char('E'))),
                opt(alt((char('+'), char('-')))),
                cut(digit1)
            )))
    )))))(input)
}

// parse a single 0 OR a non-zero digit followed by a 0 or more digits
fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(tuple((opt(one_of("+-")),
        alt((recognize(
                tuple((
                    one_of("123456789"),
                    digit0,
                )),
            ),
            tag("0")
        ))
    )))(input)
}
