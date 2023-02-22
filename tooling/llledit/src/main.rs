use std::{io::Read, path::PathBuf};

mod printer {
    #[derive(Default)]
    pub struct Printer {
        enable_alt: &'static str,
        disable_alt: &'static str,
        clear: &'static str,
        move_home: &'static str,
    }

    impl Printer {
        pub fn ansi() -> Self {
            Self {
                enable_alt: "\u{001b}[?1049h",
                disable_alt: "\u{001b}[?1049l",
                clear: "\u{001b}[2J",
                move_home: "\u{001b}[H",
            }
        }

        pub fn enable_alternate_screen(&self) {
            print!("{}", self.enable_alt);
        }

        pub fn disable_alternate_screen(&self) {
            print!("{}", self.disable_alt);
        }

        pub fn clear(&self) {
            print!("{}", self.clear);
        }

        pub fn move_home(&self) {
            print!("{}", self.move_home);
        }
    }
}
use printer::Printer;

type ErrMsg = &'static str;

type Flags = u16;

type FlagIndex = u8;

#[derive(Clone, Copy, Debug)]
enum FlagsCommand {
    Set(FlagIndex),
// TODO
    //Toggle(FlagIndex),
    //Unset(FlagIndex),
    //EditLexeme,
    //FinishedFlags,
}

fn parse_flags_commands(input: &str) -> Result<Box<[FlagsCommand]>, ErrMsg> {
    use FlagsCommand::*;

    enum ParseState {
        Initial,
        SetIndex,
    // TODO
        //ToggleIndex,
        //UnsetIndex,
    }
    use ParseState::*;

    let mut state = Initial;

    // Commands can be packed as tightly as two bytes: 's0'
    // But often they would take up more: 's10,11'
    // So half the input length is a (generous) upper bound.
    let mut output = Vec::with_capacity(input.len() / 2);

    const MAX_INDEX_DIGITS: u32 = FlagIndex::MAX.ilog10() + 1;
    let mut digit_buffer = [0u8; MAX_INDEX_DIGITS as _];
    let mut digit_buffer_i = 0;

    for c in input.chars() {
        macro_rules! push_buffered {
            ($command_fn: path) => ({
                if digit_buffer_i == 0 {
                    // Nothing to push yet.
                } else {
                    let s = std::str::from_utf8(&digit_buffer[..digit_buffer_i])
                        .map_err(|_| "non-UTF8 digit_buffer")?;

                    let flag_index =
                        s
                        .parse()
                        .map_err(|_| "non-decimal-digit numeral byte in digit_buffer")?;

                    output.push($command_fn(flag_index));

                    digit_buffer_i = 0;
                }
            })
        }

        macro_rules! push_buffered_if_needed {
            () => ({
                match state {
                    Initial => {},
                    SetIndex => {
                        push_buffered!(Set);
                    }
                }
            })
        }


        match c {
            ','|' '|'\n' => push_buffered_if_needed!(),
            's' => {
                push_buffered_if_needed!();
                state = SetIndex;
            }
            '0'..='9' => match state {
                Initial => {
                    return Err("Unexpected digit before");
                },
                SetIndex => {
                    digit_buffer[digit_buffer_i] = c.try_into()
                        .expect("should be in 0-9");
                    digit_buffer_i += 1;
                }
            }
            _ => {
                if c < ' ' {
                    return Err("got unexpected char less than ' '");
                } else if c > '\u{7f}' {
                    return Err("got unexpected char more than \\u{{7f}}");
                } else {
                    return Err("got unexpected char between ' ' and \\u{{7f}} inclusive");
                }
            }
        }
    }

    Ok(output.into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = {
        let mut args = std::env::args();
        args.next(); // exe name

        let path = args.next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("db.lll"));

        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| format!("{}: {e}", path.to_string_lossy()))?
    };

    let p = match enable_ansi_support::enable_ansi_support() {
        Ok(()) => Printer::ansi(),
        Err(e) => {
            eprintln!("{e}");
            Printer::default()
        }
    };

    p.enable_alternate_screen();
    p.clear();
    p.move_home();

    let initial_len = file.metadata()?.len();
    // Round up to nearest 256 bytes, because we expect most of the time at least
    // one lexeme will be added.
    let capacity = (initial_len | 0xFF) + 1;
    let mut lll = Vec::with_capacity(usize::try_from(capacity).unwrap_or_default());
    file.read_to_end(&mut lll)?;

    println!("{} bytes", lll.len());

    const MAX_LEXEME_LENGTH: u8 = 127;
    const MAX_LEXEME_LENGTH_ERROR: ErrMsg = "Lexemes canot be more than 127 bytes long!";
    let mut input = String::with_capacity(usize::from(MAX_LEXEME_LENGTH));

    struct Lexeme([u8; MAX_LEXEME_LENGTH as _]);

    impl Default for Lexeme {
        fn default() -> Self {
            Self([0; MAX_LEXEME_LENGTH as _])
        }
    }

    impl std::fmt::Display for Lexeme {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match std::str::from_utf8(&self.0) {
                Ok(s) => write!(f, "{s}"),
                Err(e) => write!(f, "{e}"),
            }

        }
    }

    impl TryFrom<&str> for Lexeme {
        type Error = ErrMsg;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            if value.is_empty() {
                Err("")
            } else if value.len() > usize::from(MAX_LEXEME_LENGTH) {
                Err(MAX_LEXEME_LENGTH_ERROR)
            } else {
                let mut lexeme = [0; MAX_LEXEME_LENGTH as _];

                for (i, b) in value.as_bytes().iter().enumerate() {
                    lexeme[i] = *b;
                }

                Ok(Lexeme(lexeme))
            }
        }
    }

    // Labelled Lexeme
    #[derive(Default)]
    struct LL {
        lexeme: Lexeme,
        flags: Flags
    }

    enum State {
        Menu,
        AddChars{ ll: LL },
        AddFlags{ ll: LL },
    }

    let mut state = State::Menu;
    let mut err: ErrMsg = "";

    let stdin = std::io::stdin();
    loop {
        p.clear();
        p.move_home();

        match state {
            State::Menu => {
                println!("a) Add a lexeme");
                println!("q then enter to quit");
                println!("{err}");
            }
            State::AddChars{ ref mut ll } => {
                println!("Add a lexeme");
                println!();
                println!("{err}");
                print!(">{}", ll.lexeme);
            }
            State::AddFlags{ ref mut ll } => {
                println!("Add flags to");
                println!("{}", ll.lexeme);
                println!("{:#b}", ll.flags);
                println!("To change the flags pick a operation prefix:");
                println!("s) Set bits. t) Toggle bits. u) Un-set bits.");
                println!("... then enter it followed by a comma-separated");
                println!("list of bit indexes and/or names.");
                println!("e) Edit lexeme. f) Finished editing flags.");
                println!("{err}");
            }
        }

        input.clear();
        if let Err(e) = stdin.read_line(&mut input) {
            eprintln!("{e}");
            // Do the cleanup, instead of just exiting.
            break
        }

        state = match state {
            State::Menu => {
                match input.chars().next() {
                    Some('q') => break,
                    Some('a') => {
                        err = "";
                        State::AddChars{ ll: <_>::default() }
                    },
                    None => {
                        err = "Type a letter to select an option";
                        state
                    },
                    _ => {
                        err = "???";
                        state
                    }
                }
            }
            State::AddChars{ mut ll } => {
                // TODO? Check if lexeme is already in the lll?
                match Lexeme::try_from(input.as_str()) {
                    Ok(lexeme) => {
                        ll.lexeme = lexeme;
                        err = "";
                        State::AddFlags{ ll }
                    },
                    Err(e) => {
                        err = e;
                        State::AddChars{ ll }
                    }
                }
            }
            State::AddFlags{ mut ll } => {
                match parse_flags_commands(&input) {
                    Ok(commands) => {
                        for command in commands.iter() {
                            use FlagsCommand::*;
                            match *command {
                                Set(index) => {
                                    let flag: Flags = 1 << (index as Flags);
                                    ll.flags |= flag;
                                }
                            }
                        }
                        State::AddFlags{ ll }
                    },
                    Err(e) => {
                        err = e;
                        State::AddFlags{ ll }
                    }
                }
            }
        }
    }

    p.disable_alternate_screen();

    Ok(())
}
