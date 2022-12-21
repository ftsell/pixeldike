//!
//! Custom nom combinators
//!

use nom::error::ParseError;
use nom::Parser;

/// applies the second parser on the remaining input if the first parser succeeds
pub(super) fn cond_parse<I, O1, O2, E>(
    mut first: impl Parser<I, O1, E>,
    mut second: impl Parser<I, O2, E>,
) -> impl Parser<I, O2, E>
where
    E: ParseError<I>,
{
    move |input: I| {
        let (input, _) = first.parse(input)?;
        second.parse(input)
    }
}
