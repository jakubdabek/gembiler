#[derive(Debug, Ord, PartialEq)]
struct Bruh<'a> {
    mystring: &'a str
}

trait Largest<T> {
    fn largest(self) -> Option<T>;
}

impl <T: PartialOrd, U: IntoIterator<Item=T>> Largest<T> for U {
    fn largest(self) -> Option<T> {
        let mut iter = self.into_iter();
        let mut largest = iter.next()?;

        for item in iter {
            if item > largest {
                largest = item;
            }
        }
    
        Some(largest)
    }
}

fn main() {
    let mut bruh = Bruh { mystring: "adsf" };

    println!("{:?}", bruh);

    bruh.mystring = "what";

    println!("{:?}", bruh);

    let ref mut numbers = vec![34, 50, 25, 100, 64];
    let result = numbers.largest().unwrap();
    println!("The largest number is {}", result);
    *result += 10;
    println!("Numbers are now {:?}", numbers);

    let ref chars = vec!['y', 'm', 'a', 'q'];
    let result = chars.largest().unwrap();
    println!("The largest char is {}", result);

    let f = move || { println!("{:?}", bruh); };

    f();
    f();

    // println!("{:?}", bruh);
}
