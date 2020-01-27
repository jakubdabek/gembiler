use gembiler::code_generator::{intermediate, translator};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use virtual_machine::instruction::InstructionListPrinter;

fn compile<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, output_path: P2) {
    let program = parser::parse_file(path);

    match program {
        Ok(program) => {
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
        }
        Err(err) => println!("{}", err),
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let len = args.len();

    match len {
        len if len < 3 => println!("Usage: {} <input> <output>", args[0]),
        _ => compile(args[1].as_str(), args[2].as_str()),
    }
}
