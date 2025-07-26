pub trait Colorize {
    fn light_blue(&self) -> String;

    fn green(&self) -> String;

    fn bright_red(&self) -> String;

    fn bright_yellow(&self) -> String;

    fn bright_purple(&self) -> String;
}

impl Colorize for str {
    fn light_blue(&self) -> String {
        format!("\x1b[38;5;117m{self}\x1b[0m")
    }

    fn green(&self) -> String {
        format!("\x1b[38;5;120m{self}\x1b[0m")
    }

    fn bright_red(&self) -> String {
        format!("\x1b[38;5;196m{self}\x1b[0m")
    }

    fn bright_yellow(&self) -> String {
        format!("\x1b[38;5;226m{self}\x1b[0m")
    }

    fn bright_purple(&self) -> String {
        format!("\x1b[38;5;129m{self}\x1b[0m")
    }
}
