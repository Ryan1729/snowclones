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
    println!("q then enter to quit");

    const MAX_LEXEME_LENGTH: u8 = 127;
    let mut input = String::with_capacity(usize::from(MAX_LEXEME_LENGTH));

    let stdin = std::io::stdin();
    loop {
        input.clear();
        if let Err(e) = stdin.read_line(&mut input) {
            eprintln!("{e}");
            // Do the cleanup, instead of just exiting.
            break
        }

        if input.starts_with('q') {
            break
        }
    }

    p.disable_alternate_screen();

    Ok(())
}
