use core::task::Poll;

/// Trait for building parsers.
///
/// See [`Parse`] for more details on parsers. Because parsers can hold intermediate state while
/// they parse values, it's desirable to have a state which represents a "clean" parser which has
/// not yet received any input. This trait encapsulates that state.
pub trait BuildParser<Input: Default + PartialEq> {
    /// Parser for the value.
    type Parser: Parse<Input>;

    /// Creates a new parser.
    fn build_parser(&self) -> Self::Parser;

    /// Creates a parser to parse a single value, failing on insufficient input.
    #[inline]
    fn parse_one(
        &self,
        input: Input,
    ) -> Result<
        (<Self::Parser as Parse<Input>>::Output, Input),
        <Self::Parser as Parse<Input>>::Error,
    > {
        self.build_parser().parse(input)
    }

    /// Creates a parser to parse a single value, failing on insufficient or extraneous input.
    #[inline]
    fn parse_one_only(
        &self,
        input: Input,
    ) -> Result<<Self::Parser as Parse<Input>>::Output, <Self::Parser as Parse<Input>>::Error> {
        self.build_parser().parse_only(input)
    }
}

/// Something that can parse an `Input` (usually [`str`] slices) into an `Output`.
///
/// Parsers do not have to accept all their input at once, and can instead receive input as a stream
/// over time. Additionally, they can choose to "cut off" their input at a reasonable time to allow
/// parsing a separate value, rather than just treating the whole batch of input as invalid. The
/// ways in which a parser chooses to do this are entirely up to the implementer, but these features
/// are left in as a way to allow easily combining parsers in useful ways.
///
/// For example, a parser which parses integers of a particular radix may choose to cut off all
/// input at any non-digit character, allowing the caller to pass this input to a separate parser to
/// parse a different value. However, this parser has a choice to make when it gets to a digit that
/// would overflow the returned value: it could either return an error indicating the overflow, or
/// simply cut off the input and return the excess. When passing this extra input to [`extraneous`],
/// it should be able to tell the difference, but it's up to the implementer whether they should cut
/// off only on invalid input or on all overflows as well.
///
/// [`extraneous`]: Parse::extraneous
pub trait Parse<Input: Default + PartialEq>: Sized {
    /// Output of the parser.
    type Output;

    /// Error that can occur when parsing.
    type Error;

    /// Returns an error indicating extraneous input.
    ///
    /// May panic or return weird results if given empty input.
    ///
    /// This exists mostly for the default implementations of [`try_parse_only`] and [`parse_only`],
    /// where the extraneous input is directly converted into an error.
    ///
    /// [`try_parse_only`]: Parse::try_parse_only
    /// [`parse_only`]: Parse::parse_only
    fn extraneous(&self, input: Input) -> Self::Error;

    /// Returns an error indicating insufficient input.
    ///
    /// This exists mostly for the default implementations of [`parse`] and [`parse_only`], and
    /// allows the parser to inspect its current state to provide a more helpful error about what
    /// kind of input was expected next.
    ///
    /// [`parse`]: Parse::parse
    /// [`parse_only`]: Parse::parse_only
    fn insufficient(&self) -> Self::Error;

    /// Tries to parse a value, allowing insufficient or extraneous input.
    ///
    /// If the parser can consume all of the input but not yet create a valid value
    /// (i.e., insufficient input), [`Poll::Pending`] is returned to indicate this. However, the
    /// parser does not have to parse all the input given to it and can choose to cut off early if
    /// doing so would prevent an error. To reflect this premature "cutting off" of the input, the
    /// unparsed input is returned alongside the output value. If all the input is consumed, the
    /// remaining input should be equal to [`Input::default()`].
    ///
    /// [`Input::default()`]: Default::default()
    fn try_parse(&mut self, input: Input) -> Result<Poll<(Self::Output, Input)>, Self::Error>;

    /// Parses a value, rejecting insufficient but allowing extraneous input.
    ///
    /// The parser does not have to parse all the input given to it and can choose to cut off early
    /// if doing so would prevent an error. To reflect this premature "cutting off" of the input,
    /// the unparsed input is returned alongside the output value. If all the input is consumed, the
    /// remaining input should be equal to [`Input::default()`].
    ///
    /// [`Input::default()`]: Default::default()
    fn parse(&mut self, input: Input) -> Result<(Self::Output, Input), Self::Error> {
        match self.try_parse(input)? {
            Poll::Pending => Err(self.insufficient()),
            Poll::Ready(value) => Ok(value),
        }
    }

    /// Tries to parse a value, allowing insufficient but rejecting extraneous input.
    ///
    /// If the parser can consume all of the input but not yet create a valid value
    /// (i.e., insufficient input), [`Poll::Pending`] is returned to indicate this.
    fn try_parse_only(&mut self, input: Input) -> Result<Poll<Self::Output>, Self::Error> {
        match self.try_parse(input)? {
            Poll::Pending => Ok(Poll::Pending),
            Poll::Ready((output, input)) => {
                if input == Input::default() {
                    Ok(Poll::Ready(output))
                } else {
                    Err(self.extraneous(input))
                }
            }
        }
    }

    /// Parses a value, rejecting insufficient or extraneous input.
    fn parse_only(&mut self, input: Input) -> Result<Self::Output, Self::Error> {
        match self.try_parse(input)? {
            Poll::Pending => Err(self.insufficient()),
            Poll::Ready((output, input)) => {
                if input == Input::default() {
                    Ok(output)
                } else {
                    Err(self.extraneous(input))
                }
            }
        }
    }
}
