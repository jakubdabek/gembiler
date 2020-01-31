use gembiler::code_generator::{intermediate, translator};
use std::env;
use std::fs::File;
use std::io::{Write as _};
use std::fmt::{self, Write as _, Display, Formatter, Debug};
use std::path::Path;
use virtual_machine::instruction::InstructionListPrinter;
use gembiler::verifier;

fn compile<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, output_path: P2) -> Result<(), String> {
    let program = parser::parse_file(path);

    program.and_then(|program| {
        let program = verifier::verify(program).map_err(|errors| {
            let mut buf = String::with_capacity(errors.len() * 40);

            for e in errors {
                writeln!(&mut buf, "{}", e).unwrap();
            }

            buf
        })?;

        let context = intermediate::generate(&program).unwrap();
        let generator = translator::Generator::new(context);
        let translated = generator.translate();

        let display = output_path.as_ref().display();
        let mut file = match File::create(&output_path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        file.write_fmt(format_args!(
            "{}",
            InstructionListPrinter(translated.as_slice())
        ))
        .expect("writing to file failed");

        Ok(())
    })
}

struct DebugDisplayWrapper<T: Display>(T);

impl<T: Display> Debug for DebugDisplayWrapper<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Display> From<T> for DebugDisplayWrapper<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

fn main() -> Result<(), DebugDisplayWrapper<String>> {
    let args: Vec<_> = env::args().collect();
    let len = args.len();

    match len {
        len if len < 3 => Err(format!("Usage: {} <input> <output>", args[0]).into()),
        _ => match compile(args[1].as_str(), args[2].as_str()) {
            Ok(_) => {
                println!("Output written to {}", args[2]);
                Ok(())
            },
            Err(e) => {
                Err(format!("{}", e).into())
            }
        },
    }
}
