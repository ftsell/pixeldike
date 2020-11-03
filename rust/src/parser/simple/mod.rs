use super::command::*;
use nom::combinator::{eof, opt, value};
use nom::sequence::preceded;
use nom::IResult;

mod command;
mod coordinate;
mod hex_color;

pub fn parse(input: &str) -> IResult<&str, Command> {
    // TODO Refactor this to be more readable. Maybe use combinators better.

    let (input, primary_command) = command::primary_command(input)?;
    let (input, cmd) = match primary_command {
        command::PrimaryCommand::Size => (input, Command::Size),
        command::PrimaryCommand::Help => {
            if input == "" {
                (input, Command::Help(HelpTopic::General))
            } else {
                let (input, help_topic) =
                    preceded(command::whitespace, command::help_topic)(input)?;
                (input, Command::Help(help_topic))
            }
        }
        command::PrimaryCommand::Px => {
            let (input, x) = preceded(command::whitespace, coordinate::coordinate)(input)?;
            let (input, y) = preceded(command::whitespace, coordinate::coordinate)(input)?;
            let (input, color) = opt(preceded(command::whitespace, hex_color::hex_color))(input)?;

            match color {
                None => (input, Command::PxGet(x, y)),
                Some(color) => (input, Command::PxSet(x, y, color)),
            }
        }
    };

    // if no data remains to parse => return the parsed command
    value(cmd, eof)(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    #[test]
    fn parse_size() {
        let cmd = "size";
        assert_eq!(parse(cmd), Ok(("", Command::Size)))
    }

    #[test]
    fn parse_help() {
        let cmd = "help";
        assert_eq!(parse(cmd), Ok(("", Command::Help(HelpTopic::General))));

        let cmd = "help help";
        assert_eq!(parse(cmd), Ok(("", Command::Help(HelpTopic::General))));

        let cmd = "help size";
        assert_eq!(parse(cmd), Ok(("", Command::Help(HelpTopic::Size))));

        let cmd = "help px";
        assert_eq!(parse(cmd), Ok(("", Command::Help(HelpTopic::Px))));

        let cmd = "help state";
        assert_eq!(parse(cmd), Ok(("", Command::Help(HelpTopic::State))));
    }

    quickcheck! {
        fn parse_px_get(x: usize, y: usize) -> TestResult {
            let cmd = format!("px {} {}", x, y);
            TestResult::from_bool(parse(&cmd) == Ok(("", Command::PxGet(x, y))))
        }
    }

    quickcheck! {
        fn parse_px_set(x: usize, y: usize) -> TestResult {
            // TODO make color a quickcheck parameter
            let color = Color(171, 171, 171);
            let cmd = format!("px {} {} ababab", x, y);
            TestResult::from_bool(parse(&cmd) == Ok(("", Command::PxSet(x, y, color))))
        }
    }
}
