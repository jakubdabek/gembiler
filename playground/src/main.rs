struct Container(Vec<(i32, i32)>);

impl Container {
    fn add(&mut self, value: (i32, i32), first: bool) -> &i32 {
        self.0.push(value);
        let last = self.0.last().unwrap();
        if first {
            &last.0
        } else {
            &last.1
        }
    }
}

struct ContainerContainer<'a> {
    container: Container,
    transformed: Vec<&'a i32>,
}

impl <'a> ContainerContainer<'a> {
    fn new() -> Self {
        ContainerContainer {
            container: Container(vec![]),
            transformed: vec![],
        }
    }

    fn add(&'a mut self, value: i32) {
        let value = self.container.add((value, value), value > 10);
        self.transformed.push(&value);
    }
}

use easybench::bench;
use rand::prelude::*;


fn flatmap(inp: &Vec<i64>) -> Vec<i64> {
    inp.iter().flat_map(|elem| {
        1..*elem
    }).collect()
}

fn mutvec(inp: &Vec<i64>) -> Vec<i64> {
    let mut vec = vec![];

    for elem in inp {
        for i in 1..*elem {
            vec.push(i);
        }
    }

    vec
}

fn main() {
    let mut container_container = ContainerContainer::new();
    container_container.add(5);

    let mut v = vec![1,2,3,4,5];
    let mut iter = v.iter_mut();
    let v1 = iter.next().unwrap();
    let v2 = iter.next().unwrap();

    *v1 += 10;
    *v2 += 10;

    println!("{}{}", v1, v2);
    println!("{:?}", v);

    let mut rng = thread_rng();

    let input: Vec<i64> = std::iter::repeat_with(|| rng.gen_range(1, 200)).take(50).collect();

    println!("flatmap: {}", bench(|| flatmap(&input)));
    println!("mutvec: {}", bench(|| mutvec(&input)));
}
