use std::{fs::File, io::{self, Read}, path::PathBuf};

macro_rules! compile_time_assert {
    ($assertion: expr) => {{
        #[allow(unknown_lints, eq_op)]
        // Based on the const_assert macro from static_assertions;
        const _: [(); 0 - !{$assertion} as usize] = [];
    }}
}

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

type Flags = u32;

type FlagIndex = u8;
type AdjectiveOrderCategory = u8;

#[derive(Clone, Copy, Debug)]
enum FlagsCommand {
    Set(FlagIndex),
    Toggle(FlagIndex),
    Unset(FlagIndex),
    SetAdjectiveOrder(AdjectiveOrderCategory),
    EditLexeme,
    FinishedFlags,
}

fn parse_flags_commands(input: &str) -> Result<Box<[FlagsCommand]>, ErrMsg> {
    // We'd rather set digit_buffer_i to 0 (inside push_buffered) when it isn't
    // needed than miss setting it to 0 when we should.
    // TODO? Can we scope this attribute tighter? Abve the assingment doesn't work
    // at the moment.
    #![allow(unused_assignments)]

    use FlagsCommand::*;

    enum ParseState {
        SetIndex,
        ToggleIndex,
        UnsetIndex,
        SetAdjectiveOrderCategory,
    }
    use ParseState::*;

    let mut state = SetIndex;

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
                    SetIndex => {
                        push_buffered!(Set);
                    },
                    ToggleIndex => {
                        push_buffered!(Toggle);
                    },
                    UnsetIndex => {
                        push_buffered!(Unset);
                    },
                    SetAdjectiveOrderCategory => {
                        push_buffered!(SetAdjectiveOrder);
                    }
                }
            })
        }


        match c {
            ','|' '|'\n' => push_buffered_if_needed!(),
            's' => {
                push_buffered_if_needed!();
                state = SetIndex;
            },
            't' => {
                push_buffered_if_needed!();
                state = ToggleIndex;
            },
            'u' => {
                push_buffered_if_needed!();
                state = UnsetIndex;
            },
            '0'..='9' => {
                digit_buffer[digit_buffer_i] = c.try_into()
                    .expect("should be in 0-9");
                digit_buffer_i += 1;
            },
            'a' => {
                push_buffered_if_needed!();
                state = SetAdjectiveOrderCategory;
            },
            'e' => {
                push_buffered_if_needed!();
                output.push(FlagsCommand::EditLexeme);
                break
            },
            'f' => {
                push_buffered_if_needed!();
                output.push(FlagsCommand::FinishedFlags);
                break
            },
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

const V0_HEADER: [u8; 4] = [b'l', b'l', b'l', 0];
const V0_MIN_LENGTH: u8 = 4;

mod lexeme {
    use super::*;

    pub const MAX_LENGTH: u8 = 127 - V0_MIN_LENGTH;
    pub const MAX_LENGTH_ERROR: ErrMsg = "Lexemes canot be more than 123 bytes long!";

    #[derive(Clone)]
    pub struct Lexeme([u8; MAX_LENGTH as _]);

    impl Default for Lexeme {
        fn default() -> Self {
            Self([0; MAX_LENGTH as _])
        }
    }

    impl std::fmt::Display for Lexeme {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match std::str::from_utf8(&self.0) {
                Ok(s) => if f.alternate() {
                    write!(f, "\"{s}\"")
                } else {
                    write!(f, "{s}")
                },
                Err(e) => write!(f, "{e}"),
            }

        }
    }

    impl TryFrom<&[u8]> for Lexeme {
        type Error = ErrMsg;

        fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            // Pass through str so we are sure that all `Lexeme`s are valid UTF-8
            match std::str::from_utf8(value) {
                Err(_) => Err("Potential lexeme was not valid UTF-8"),
                Ok(s) => Self::try_from(s)
            }
        }
    }

    impl TryFrom<&str> for Lexeme {
        type Error = ErrMsg;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            let value = value.trim();

            if value.is_empty() {
                Err("")
            } else if value.len() > usize::from(MAX_LENGTH) {
                Err(MAX_LENGTH_ERROR)
            } else {
                let mut lexeme = [0; MAX_LENGTH as _];

                for (i, b) in value.as_bytes().iter().enumerate() {
                    lexeme[i] = *b;
                }

                Ok(Lexeme(lexeme))
            }
        }
    }

    impl Lexeme {
        pub fn len(&self) -> u8 {
            self.as_str().len() as u8
        }

        pub fn bytes(&self) -> &[u8] {
            self.as_str().as_bytes()
        }

        fn as_str(&self) -> &str {
            // This module only exposes ways to create `Lexeme`s that ensure this
            // cannot fail.
            std::str::from_utf8(&self.0)
                .expect("all lexemes should be valid UTF-8")
                .trim_end_matches('\0')
        }
    }
}
use lexeme::Lexeme;

// Labelled Lexeme
#[derive(Clone, Default)]
struct LL {
    lexeme: Lexeme,
    flags: Flags
}

fn parse_lll(bytes: &[u8]) -> Result<Vec<LL>, ErrMsg> {
    if bytes.len() < V0_HEADER.len() {
        return Err("File was not .lll format: Too short.");
    }
    if bytes[0] != V0_HEADER[0]
    || bytes[1] != V0_HEADER[1]
    || bytes[2] != V0_HEADER[2] {
        return Err("File was not .lll format: Header wrong");
    }

    if bytes[3] != 0 {
        if bytes[3] == 1 {
            return Err("File was an unsupported version of .lll format: 1");
        } else if bytes[3] == 2 {
            return Err("File was an unsupported version of .lll format: 2");
        } else {
            return Err("File was an unsupported version of .lll format: >2");
        }
    }

    const BLOCK_HEADER_LENGTH: u8 = 4;

    let mut output = Vec::with_capacity(bytes.len() / 16);

    let mut i = V0_HEADER.len();
    while i < bytes.len() {
        let len = bytes[i] & 0x7F;
        if len < BLOCK_HEADER_LENGTH {
            return Err("Table seems corrupted: Block length was invalid.");
        }

        let block_end = i + usize::from(len);
        if block_end > bytes.len() {
            break
        }

        let fef = (bytes[i] & 0x80) == 0x80;

        if fef {
            // Skip because we don't know what the FEF does yet.
            i = block_end;
            continue
        }

        let lexeme = (&bytes[
            i + usize::from(BLOCK_HEADER_LENGTH)..block_end
        ]).try_into()?;

        let flags = bytes[i + 1] as Flags
            | (bytes[i + 2] as Flags) << 8
            | (bytes[i + 3] as Flags) << 16
            ;

        output.push(LL {
            lexeme,
            flags
        });

        i = block_end;
    }

    Ok(output)
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

    let initial_len = file.metadata()?.len();
    // Round up to nearest 256 bytes, because we expect most of the time at least
    // one lexeme will be added.
    let capacity = (initial_len | 0xFF) + 1;
    let mut bytes = Vec::with_capacity(usize::try_from(capacity).unwrap_or_default());
    file.read_to_end(&mut bytes)?;

    let mut lll: Vec<LL> = parse_lll(&bytes)?;

    p.enable_alternate_screen();
    p.clear();
    p.move_home();

    let mut input = String::with_capacity(usize::from(lexeme::MAX_LENGTH));

    enum State {
        Menu,
        AddChars{ ll: LL },
        AddFlags{ ll: LL },
        SelectEditIndex{ index: Option<usize> },
        EditChars{ ll: LL, index: usize },
        EditFlags{ ll: LL, index: usize },
    }

    let mut state = State::Menu;
    let mut err: ErrMsg = "";

    let stdin = std::io::stdin();
    loop {
        macro_rules! break_if_err {
            ($res: expr) => {
                if let Err(e) = $res {
                    eprintln!("{e}");
                    // Do the cleanup, instead of just exiting.
                    break
                }
            }
        }

        p.clear();
        p.move_home();

        const FLAG_NAMES: [&str; 16] = [
            "SINGULAR_NOUN",
            "PLURAL_NOUN",
            "MASS_NOUN",
            "RESERVED",
            "INTRANSITIVE_VERB",
            "TRANSITIVE_VERB",
            "RESERVED",
            "RESERVED",
            "THIRD_PERSON_SINGULAR_VERB",
            "RESERVED",
            "FIRST_PERSON_SINGULAR_VERB",
            "RESERVED",
            "RESERVED",
            "RESERVED",
            "RESERVED",
            "RESERVED",
        ];

        // Used in compile-time asserts.
        #[allow(dead_code)]
        const MAX_FLAG_NAME_LEN: usize = {
            let mut max_len = 0;
            let mut i = 0;
            while i < FLAG_NAMES.len() {
                let len = FLAG_NAMES[i].len();
                if len > max_len {
                    max_len = len;
                }
                i += 1;
            }
            max_len
        };

        const ADJECTIVE_ORDER_BLOCK_NAMES: [&str; 16] = [
            "NONE",
            "RESERVED",
            "RESERVED",
            "RESERVED",
            "QUANTITY",
            "OBSERVATION",
            "SIZE",
            "PHYSICAL",
            "SHAPE",
            "AGE",
            "COLOUR",
            "ORIGIN",
            "MATERIAL",
            "RESERVED",
            "RESERVED",
            "RESERVED",
        ];

        // Used in compile-time asserts.
        #[allow(dead_code)]
        const MAX_ADJECTIVE_ORDER_BLOCK_NAME_LEN: usize = {
            let mut max_len = 0;
            let mut i = 0;
            while i < ADJECTIVE_ORDER_BLOCK_NAMES.len() {
                let len = ADJECTIVE_ORDER_BLOCK_NAMES[i].len();
                if len > max_len {
                    max_len = len;
                }
                i += 1;
            }
            max_len
        };

        match state {
            State::Menu => {
                println!("a) Add a lexeme");
                println!("e) Edit a lexeme");
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
                println!("{:#}", ll.lexeme);
                println!(">{:#b}", ll.flags);
                println!("To change the flags pick a operation prefix:");
                println!("s) Set bits. t) Toggle bits. u) Un-set bits.");
                println!("... then enter it followed by a comma-separated");
                println!("list of bit indexes.");
                println!("To change a block value enter a block prefix:");
                println!("a) adjective order.");
                println!("... then enter it followed by the desired value.");
                println!("");
                println!("Flags:");
                {
                    let half_len = (FLAG_NAMES.len() + 1) / 2;

                    for i in 0..half_len {
                        let first_name = FLAG_NAMES[i];
                        let i2 = half_len + i;
                        if let Some(second_name) = FLAG_NAMES.get(i2) {
                            // assert format width is large enough
                            compile_time_assert!(
                                30 >= MAX_FLAG_NAME_LEN
                            );
                            println!("{first_name:>30}:{i:2} {second_name:>30}:{i2:2}");
                        } else {
                            println!("{first_name:>30}:{i:2}");
                        }
                    }
                }
                println!("Adjective Order Block:");
                {
                    let half_len = (ADJECTIVE_ORDER_BLOCK_NAMES.len() + 1) / 2;

                    for i in 0..half_len {
                        let first_name = ADJECTIVE_ORDER_BLOCK_NAMES[i];
                        let i2 = half_len + i;
                        if let Some(second_name) = ADJECTIVE_ORDER_BLOCK_NAMES.get(i2) {
                            // assert format width is large enough
                            compile_time_assert!(
                                30 >= MAX_ADJECTIVE_ORDER_BLOCK_NAME_LEN
                            );
                            println!("{first_name:>30}:{i:2} {second_name:>30}:{i2:2}");
                        } else {
                            println!("{first_name:>30}:{i:2}");
                        }
                    }
                }
                println!("e) Edit lexeme. f) Finished editing flags.");
                println!("{err}");
            }
            State::SelectEditIndex{ ref mut index } => {
                println!("Select a lexeme");
                println!("Enter an index, or");
                println!("q) go back to the menu");
                match index.and_then(|i| lll.get(i).map(|ll| (i, ll))) {
                    Some((i, ll)) => {
                        println!("e) edit this lexeme f) edit this lexeme's flags");
                        println!();
                        println!("{err}");
                        println!("@{}", i);
                        // TODO print the surrounding lexemes in the lll
                        println!("{:#}", ll.lexeme);
                        println!("{:#b}", ll.flags);
                    },
                    None => {
                        println!();
                        println!("{err}");
                        print!(">");
                    }
                }

            }
            State::EditChars{ ref mut ll, index } => {
                println!("Edit a lexeme");
                println!();
                println!("{err}");
                if let Some(prev) = lll.get(index) {
                    println!("{:#}", prev.lexeme);
                    println!();
                }
                print!(">{}", ll.lexeme);
            }
            State::EditFlags{ ref mut ll, index } => {
                println!("Edit flags for");
                println!("{:#}", ll.lexeme);
                if let Some(prev) = lll.get(index) {
                    println!("{:#b}", prev.flags);
                    println!();
                }
                println!(">{:#b}", ll.flags);
                println!("To change the flags pick a operation prefix:");
                println!("s) Set bits. t) Toggle bits. u) Un-set bits.");
                println!("... then enter it followed by a comma-separated");
                println!("list of bit indexes.");
                println!("To change a block value enter a block prefix:");
                println!("a) adjective order.");
                println!("... then enter it followed by the desired value.");
                println!("");
                println!("Flags:");
                {
                    let half_len = (FLAG_NAMES.len() + 1) / 2;

                    for i in 0..half_len {
                        let first_name = FLAG_NAMES[i];
                        let i2 = half_len + i;
                        if let Some(second_name) = FLAG_NAMES.get(i2) {
                            // assert format width is large enough
                            compile_time_assert!(
                                30 >= MAX_FLAG_NAME_LEN
                            );
                            println!("{first_name:>30}:{i:2} {second_name:>30}:{i2:2}");
                        } else {
                            println!("{first_name:>30}:{i:2}");
                        }
                    }
                }
                println!("Adjective Order Block:");
                {
                    let half_len = (ADJECTIVE_ORDER_BLOCK_NAMES.len() + 1) / 2;

                    for i in 0..half_len {
                        let first_name = ADJECTIVE_ORDER_BLOCK_NAMES[i];
                        let i2 = half_len + i;
                        if let Some(second_name) = ADJECTIVE_ORDER_BLOCK_NAMES.get(i2) {
                            // assert format width is large enough
                            compile_time_assert!(
                                30 >= MAX_ADJECTIVE_ORDER_BLOCK_NAME_LEN
                            );
                            println!("{first_name:>30}:{i:2} {second_name:>30}:{i2:2}");
                        } else {
                            println!("{first_name:>30}:{i:2}");
                        }
                    }
                }
                println!("e) Edit lexeme. f) Finished editing flags.");
                println!("{err}");
            }
        }

        input.clear();
        break_if_err!(stdin.read_line(&mut input));

        state = match state {
            State::Menu => {
                match input.chars().next() {
                    Some('q') => break,
                    Some('a') => {
                        err = "";
                        State::AddChars{ ll: <_>::default() }
                    },
                    Some('e') => {
                        err = "";
                        State::SelectEditIndex{ index: None }
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
                        let switch = handle_commands(&mut ll, &commands);

                        match switch {
                            StateSwitch::Stay => {
                                State::AddFlags{ ll }
                            },
                            StateSwitch::EditLexeme => {
                                State::AddChars{ ll }
                            },
                            StateSwitch::Finished => {
                                lll.push(ll);

                                break_if_err!(write_lll_to_disk(
                                    &mut file,
                                    &lll
                                ));

                                State::Menu
                            },
                        }
                    },
                    Err(e) => {
                        err = e;
                        State::AddFlags{ ll }
                    }
                }
            }
            State::SelectEditIndex{ ref mut index } => {
                match (*index, input.chars().next()) {
                    (_, Some('q')) => {
                        err = "";
                        State::Menu
                    },
                    (Some(i), Some('e')) => {
                        match lll.get(i) {
                            Some(ll) => {
                                err = "";
                                State::EditChars{ ll: ll.clone(), index: i }
                            }
                            None => {
                                err = "No lexeme at that index";
                                State::SelectEditIndex{ index: Some(i) }
                            }
                        }
                    },
                    (Some(i), Some('f')) => {
                        match lll.get(i) {
                            Some(ll) => {
                                err = "";
                                State::EditFlags{ ll: ll.clone(), index: i }
                            }
                            None => {
                                err = "No lexeme at that index";
                                State::SelectEditIndex{ index: Some(i) }
                            }
                        }
                    },
                    _ => {
                        dbg!(input.as_str());
                        // TODO? allow jumping to add a new lexeme from here?
                        match usize::from_str_radix(input.as_str().trim(), 10) {
                            Ok(i) => {
                                *index = Some(i);
                                err = "";
                                State::SelectEditIndex{ index: *index }
                            },
                            Err(_) => {
                                err = "Could not parse index";
                                State::SelectEditIndex{ index: *index }
                            }
                        }
                    }
                }
            }
            State::EditChars{ mut ll, index } => {
                // TODO? Implement actual piecewise editing,
                // instead of just replacing?
                match Lexeme::try_from(input.as_str()) {
                    Ok(lexeme) => {
                        ll.lexeme = lexeme;
                        err = "";
                        State::EditFlags{ ll, index }
                    },
                    Err(e) => {
                        err = e;
                        State::EditChars{ ll, index }
                    }
                }
            }
            State::EditFlags{ mut ll, index } => {
                match parse_flags_commands(&input) {
                    Ok(commands) => {
                        let switch = handle_commands(&mut ll, &commands);

                        match switch {
                            StateSwitch::Stay => {
                                State::EditFlags{ ll, index }
                            },
                            StateSwitch::EditLexeme => {
                                State::EditChars{ ll, index }
                            },
                            StateSwitch::Finished => {
                                if let Some(_) = lll.get(index) {
                                    lll[index] = ll;
                                } else {
                                    // TODO? Break instead? check for a duplicate?
                                    lll.push(ll);
                                }

                                break_if_err!(write_lll_to_disk(
                                    &mut file,
                                    &lll
                                ));

                                State::Menu
                            },
                        }
                    },
                    Err(e) => {
                        err = e;
                        State::EditFlags{ ll, index }
                    }
                }
            }
        }
    }

    p.disable_alternate_screen();

    Ok(())
}

enum StateSwitch {
    Stay,
    EditLexeme,
    Finished,
}

fn handle_commands(ll: &mut LL, commands: &[FlagsCommand]) -> StateSwitch {
    let mut switch = StateSwitch::Stay;
    for command in commands.iter() {
        use FlagsCommand::*;
        match *command {
            Set(index) => {
                let flag: Flags = 1 << (index as Flags);
                ll.flags |= flag;
            }
            Toggle(index) => {
                let flag: Flags = 1 << (index as Flags);
                ll.flags ^= flag;
            }
            Unset(index) => {
                let flag: Flags = 1 << (index as Flags);
                ll.flags &= !flag;
            }
            SetAdjectiveOrder(category) => {
                const ADJECTIVE_ORDER_SHIFT: Flags = 16;
                ll.flags &= !(0b1111 << ADJECTIVE_ORDER_SHIFT);
                ll.flags |= (category as Flags) << ADJECTIVE_ORDER_SHIFT;
            }
            EditLexeme => {
                switch = StateSwitch::EditLexeme;
                break
            }
            FinishedFlags => {
                switch = StateSwitch::Finished;
                break
            }
        }
    }
    switch
}

fn write_lll_to_disk(file: &mut File, lll: &[LL]) -> io::Result<()> {
    use std::io::{Seek, SeekFrom, Write};

    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;

    file.write(&V0_HEADER)?;
    for ll in lll {
        file.write(&[
            V0_MIN_LENGTH + ll.lexeme.len() as u8
        ])?;

        file.write(&(ll.flags.to_le_bytes())[0..3])?;

        file.write(ll.lexeme.bytes())?;
    }

    file.flush()?;

    Ok(())
}