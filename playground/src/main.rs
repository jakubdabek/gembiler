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

fn main() {
    let mut container_container = ContainerContainer::new();
    container_container.add(5);
}
