use core::{mem, task::Poll};
use num_traits::{CheckedAdd, CheckedMul};
use parse_trait::{BuildParser, Parse};

/// Parses an integer from a string.
pub struct ParseInt<T: Default + CheckedAdd + CheckedMul> {
    /// Current parsed value.
    val: T,

    /// Radix being parsed.
    radix: T,
}

/// Error parsing an integer.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ParseIntError {
    /// Non-digit character.
    InvalidChar(char),

    /// Can't parse an empty string.
    Empty,

    /// Overflow during parsing.
    Overflow,
}

/// Generic trait to make implementing easier (a libstd version would just use macros, probably).
pub trait FromRadix: Default + CheckedAdd + CheckedMul {
    /// Equivalent to 2 <= radix && radix <= 36.
    fn is_valid_radix(radix: &Self) -> bool;

    /// Equivalent to `char::is_digit`.
    fn is_digit_radix(c: char, radix: &Self) -> bool;

    /// Equivalent to `char::to_digit`.
    fn from_digit_radix(c: char, radix: &Self) -> Self;
}
impl FromRadix for u32 {
    fn is_valid_radix(radix: &u32) -> bool {
        2 <= *radix && *radix <= 36
    }
    fn is_digit_radix(c: char, radix: &Self) -> bool {
        c.is_digit(*radix)
    }
    fn from_digit_radix(c: char, radix: &Self) -> Self {
        c.to_digit(*radix).unwrap()
    }
}

impl<'a, T: FromRadix> Parse<&'a str> for ParseInt<T> {
    type Output = T;
    type Error = ParseIntError;

    fn extraneous(&self, input: &'a str) -> Self::Error {
        match input.chars().next() {
            None => ParseIntError::Empty,
            Some(c) => {
                if T::is_digit_radix(c, &self.radix) {
                    ParseIntError::Overflow
                } else {
                    ParseIntError::InvalidChar(c)
                }
            }
        }
    }

    fn insufficient(&self) -> Self::Error {
        ParseIntError::Empty
    }

    fn try_parse(
        &mut self,
        mut input: &'a str,
    ) -> Result<Poll<(Self::Output, &'a str)>, Self::Error> {
        let remaining;
        if let Some(pos) = input.find(|c: char| !T::is_digit_radix(c, &self.radix)) {
            (input, remaining) = input.split_at(pos);
            if input.is_empty() {
                return Err(ParseIntError::InvalidChar(
                    remaining.chars().next().unwrap(),
                ));
            }
        } else if input.is_empty() {
            return Ok(Poll::Pending);
        } else {
            remaining = "";
        }

        for c in input.chars() {
            self.val = self
                .val
                .checked_mul(&self.radix)
                .ok_or(ParseIntError::Overflow)?;
            let digit = T::from_digit_radix(c, &self.radix);
            self.val = self
                .val
                .checked_add(&digit)
                .ok_or(ParseIntError::Overflow)?;
        }

        Ok(Poll::Ready((mem::take(&mut self.val), remaining)))
    }
}

/// Would likely be returned by some `T::parse_radix` method.
pub struct ParseRadix<T>(T);
impl<'a, T: Clone + FromRadix> BuildParser<&'a str> for ParseRadix<T> {
    type Parser = ParseInt<T>;
    fn build_parser(&self) -> ParseInt<T> {
        let radix = self.0.clone();
        ParseInt {
            val: T::default(),
            radix,
        }
    }
}

/// Simple example to make sure it works.
#[test]
fn deadbeef() {
    assert_eq!(ParseRadix(16).parse_one_only("deadbeef"), Ok(0xdeadbeef));
}
