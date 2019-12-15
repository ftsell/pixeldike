use crate::color::{color_from_rgba, Color};
use hex::FromHex;

pub type ParseError = String;

pub enum Command {
    ///
    /// `Help()` informs the user about how to use this protocol
    ///
    Help(),

    ///
    /// `Help(subcommand)` informs the user about how a specific subcommand works
    ///
    HelpSubcommand(String),

    ///
    /// `Size()` returns the canvas's size
    ///
    Size(),

    ///
    /// `GetPx(x, y)` queries the current color at the position described through x and y
    ///
    GetPx(usize, usize),

    ///
    /// `SetPx(x, y, color)` sets a new color to the pixel at position x and y
    ///
    SetPx(usize, usize, Color),

    ///
    /// `Binary()` gets the whole canvas as bulk binary data
    ///
    Binary(),
}

impl Command {
    ///
    /// Parse the given input string into a valid command if possible
    ///
    pub fn parse(input: &String) -> Result<Command, ParseError> {
        let input = input.to_lowercase();
        let parts: Vec<&str> = input.split(" ").collect();

        if parts[0] == "help" {
            if parts.len() == 1 {
                return Ok(Command::Help());
            } else {
                return Ok(Command::HelpSubcommand(parts[1].to_string()));
            }
        } else if parts[0] == "size" {
            return Ok(Command::Size());
        } else if parts[0] == "px" {
            if parts.len() < 3 {
                return Err("Not enough arguments to PX command".to_string());
            }

            let x = parts[1].parse::<usize>();
            if x.is_err() {
                return Err(format!(
                    "Could not interpret X parameter ({}) as number",
                    parts[1]
                ));
            }

            let y = parts[2].parse::<usize>();
            if y.is_err() {
                return Err(format!(
                    "Could not interpret Y parameter ({}) as number",
                    parts[2]
                ));
            }

            if parts.len() < 4 {
                // In this case it was only a getPX command
                return Ok(Command::GetPx(x.unwrap(), y.unwrap()));
            } else {
                // It is a setPX command

                let color = match Vec::from_hex(parts[3].to_string()) {
                    Err(_) => return Err("Could not interpret color parameter as HEX".to_string()),
                    Ok(x) => x,
                };

                let r = color.get(0);
                let g = color.get(1);
                let b = color.get(2);

                if r.is_none() || g.is_none() || b.is_none() {
                    return Err("Color parameter has incorect length".to_string());
                }

                return Ok(Command::SetPx(
                    x.unwrap(),
                    y.unwrap(),
                    color_from_rgba(*r.unwrap(), *g.unwrap(), *b.unwrap()),
                ));
            }
        } else if parts[0] == "state" {
            return Ok(Command::Binary());
        }

        return Ok(Command::Help());
    }
}
