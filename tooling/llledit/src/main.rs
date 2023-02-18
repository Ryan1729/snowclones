mod printer {
    #[derive(Default)]
    pub struct Printer {
        enable_alt: &'static str,
        disable_alt: &'static str,
        clear: &'static str,
    }

    impl Printer {
        pub fn ansi() -> Self {
            Self {
                enable_alt: "\u{001b}[?1049h",
                disable_alt: "\u{001b}[?1049l",
                clear: "\u{001b}[2J"
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
    }
}
use printer::Printer;

fn main() {
    let p = match enable_ansi_support::enable_ansi_support() {
        Ok(()) => Printer::ansi(),
        Err(e) => {
            eprintln!("{e}");
            Printer::default()
        }
    };

    p.enable_alternate_screen();
    p.clear();

    println!("Hello, world!");

    for i in (1..=3).rev() {
        println!("{i}");

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    p.disable_alternate_screen();
}
