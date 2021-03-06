#[macro_use]
extern crate lazy_static;

use rand::distributions::Uniform;
use rand::prelude::*;
use std::collections::HashMap;

lazy_static! {
    pub static ref TEST_DATA: HashMap<String, ProgramData> = get_all_programs();
}

pub struct ProgramData {
    pub text: &'static str,
    pub valid_io: Vec<(Vec<i64>, Vec<i64>)>,
}

mod data;

fn generate_program_data(data: data::Data, inputs: Vec<Vec<i64>>) -> ProgramData {
    let io = inputs.into_iter().map(|v| {
        let result = data.exec(v.clone());
        (v, result)
    });

    ProgramData {
        text: data.text(),
        valid_io: io.collect(),
    }
}

pub fn get_all_programs() -> HashMap<String, ProgramData> {
    let mut programs = HashMap::new();

    let mut rng = rand::rngs::StdRng::seed_from_u64(420);

    let dist = Uniform::new(1, 1_000_000_000);
    let dist_small = Uniform::new(1, 1_000);
    programs.insert(
        String::from("bitstring"),
        generate_program_data(data::BITSTRING_DATA, {
            let mut base = vec![vec![10], vec![1_345_601]];

            base.extend(std::iter::repeat_with(|| vec![dist.sample(&mut rng)]).take(10));

            base
        }),
    );
    programs.insert(
        String::from("sieve"),
        generate_program_data(data::SIEVE_DATA, vec![vec![]]),
    );

    programs.insert(
        String::from("sieve"),
        generate_program_data(data::SIEVE_DATA, vec![vec![]]),
    );

    programs.insert(
        String::from("prime_decomposition_small"),
        generate_program_data(data::PRIME_DECOMPOSITION_DATA, {
            let mut base = vec![
                vec![2],
                vec![3],
                vec![4],
                vec![10],
                vec![25],
                vec![27],
                vec![64],
            ];

            base.extend(std::iter::repeat_with(|| vec![dist_small.sample(&mut rng)]).take(4));

            base
        }),
    );

    programs.insert(
        String::from("prime_decomposition_large1"),
        generate_program_data(data::PRIME_DECOMPOSITION_DATA, {
            std::iter::repeat_with(|| vec![dist.sample(&mut rng)]).take(4).collect()
        }),
    );

    programs.insert(
        String::from("prime_decomposition_large2"),
        generate_program_data(data::PRIME_DECOMPOSITION_DATA, {
            vec![
                vec![123_456_543_210],
                vec![12_345_678_901],
            ]
        }),
    );

    programs.insert(
        String::from("prime_decomposition_large3"),
        generate_program_data(data::PRIME_DECOMPOSITION_DATA, {
            vec![
                vec![12_345_678_903],
            ]
        }),
    );

    programs.insert(
        String::from("div_mod"),
        generate_program_data(data::DIV_MOD_DATA, vec![vec![1, 0], vec![1, 2]]),
    );

    programs.insert(
        String::from("div_mod2"),
        generate_program_data(data::DIV_MOD2_DATA, vec![vec![1, 33, 7, 0], {
            let mut base = vec![
                1, 1, 1, 1, 1, 0, 1, 33, 7, 1, 33, 8, 1, 33, 9, 1, 33, 10, 1, 33, 11, 1, 12, 14,
            ];

            base.extend(
                std::iter::repeat_with(|| vec![1, dist.sample(&mut rng), dist.sample(&mut rng)])
                    .flatten()
                    .take(10 * 3),
            );

            base.push(0);

            base
        }]),
    );

    programs.insert(
        String::from("numbers"),
        generate_program_data(data::NUMBERS_DATA, (-20..=20).map(|i| vec![i]).collect()),
    );

    programs.insert(
        String::from("fib"),
        generate_program_data(data::FIB_DATA, (1..=20).map(|i| vec![i]).collect()),
    );

    programs.insert(
        String::from("fib_factorial"),
        generate_program_data(
            data::FIB_FACTORIAL_DATA,
            (1..=20).map(|i| vec![i]).collect(),
        ),
    );

    programs.insert(
        String::from("factorial"),
        generate_program_data(data::FACTORIAL_DATA, (1..=20).map(|i| vec![i]).collect()),
    );

    programs.insert(
        String::from("tab"),
        generate_program_data(data::TAB_DATA, vec![vec![]]),
    );

    programs.insert(
        String::from("mod_mult"),
        generate_program_data(data::MOD_MULT_DATA, vec![vec![
            1_234_567_890,
            1_234_567_890_987_654_321,
            987_654_321,
        ]]),
    );

    programs.insert(
        String::from("loopiii"),
        generate_program_data(data::LOOPIII_DATA, {
            let mut base = vec![vec![0, 0, 0], vec![1, 0, 2]];

            base.extend(
                std::iter::repeat_with(|| {
                    vec![
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                    ]
                })
                .take(10),
            );

            base
        }),
    );

//    programs.insert(
//        String::from("for_loop"),
//        generate_program_data(data::FOR_LOOP_DATA, vec![vec![12, 23, 34]]),
//    );

    programs.insert(
        String::from("ifs"),
        generate_program_data(
            data::IFS_DATA,
            (-15..=15).flat_map(|v| {
                vec![vec![v, v], vec![v, v - 1], vec![v, v + 1]].into_iter()
            }).collect()
        ),
    );

    programs
}
