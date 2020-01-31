#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Bruh<'a> {
    mystring: &'a str,
}

trait Largest<T> {
    fn largest(self) -> Option<T>;
}

impl<T: PartialOrd, U: IntoIterator<Item = T>> Largest<T> for U {
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

    let numbers = &mut vec![34, 50, 25, 100, 64];
    let result = numbers.largest().unwrap();
    println!("The largest number is {}", result);
    *result += 10;
    println!("Numbers are now {:?}", numbers);

    let chars = &vec!['y', 'm', 'a', 'q'];
    let result = chars.largest().unwrap();
    println!("The largest char is {}", result);

    let f: &dyn Fn() = &move || {
        println!("{:?}", bruh);
    };

//    let f: &mut dyn FnMut() = &mut move || {
//        println!("{:?}", bruh);
//        // drop(bruh);
//    };

    f();
    f();

    let mut f = {
        let mut bruh = Bruh { mystring: "123" };
        let f = move || {
            println!("{:?}", bruh);
            bruh.mystring = "1234";
            &mut bruh
        };
//        println!("{:?}", bruh);
        f
    };

    f();
    f();

    // println!("{:?}", bruh);
    let mut i = 123i32;
    // foo(&mut i);
    bar(&mut i);
    println!("{}", i);

    let mut i = Box::new(123);
    foo(&mut i);
    bar(&mut i);
    println!("{}", i);
}

use std::borrow::BorrowMut;

fn foo<T: AsMut<i32>>(a: &mut T) {
    *a.as_mut() += 1;
}

fn bar<T: BorrowMut<i32>>(a: &mut T) {
    *a.borrow_mut() += 1;
}

trait MyInto<T>: Sized {
    fn into(self) -> T;
}

trait MyFrom<T>: Sized {
    fn from(t: T) -> Self;
}

impl<T, U> MyFrom<U> for T
where
    U: MyInto<T>,
{
    fn from(u: U) -> T {
        u.into()
    }
}

#[cfg(test)]
mod largest_tests {
    use super::Largest;

    #[test]
    fn empty_slice() {
        let mut empty_slice: [i32; 0] = [];

        let ref_result: Option<&i32> = (&empty_slice).largest();
        assert!(ref_result.is_none());

        let ref_mut_result: Option<&mut i32> = (&mut empty_slice).largest();
        assert!(ref_mut_result.is_none());

        let value_result: Option<&i32> = empty_slice.largest();
        assert!(value_result.is_none());
    }

    #[test]
    fn empty_vec() {
        let mut empty_vec: Vec<i32> = vec![];

        let ref_result: Option<&i32> = (&empty_vec).largest();
        assert!(ref_result.is_none());

        let ref_mut_result: Option<&mut i32> = (&mut empty_vec).largest();
        assert!(ref_mut_result.is_none());

        let value_result: Option<i32> = empty_vec.largest();
        assert!(value_result.is_none());
    }

    #[test]
    fn one_element() {
        let mut vec = vec![1];

        let ref_result: Option<&i32> = (&vec).largest();
        assert_eq!(ref_result, Some(&1));

        let ref_mut_result: Option<&mut i32> = (&mut vec).largest();
        assert_eq!(ref_mut_result, Some(&mut 1));
        *ref_mut_result.unwrap() = 2;
        assert_eq!(vec[0], 2);

        let value_result: Option<i32> = vec.largest(); // vec moved
        assert_eq!(value_result, Some(2));
    }

    use rand::{seq::SliceRandom, thread_rng};

    #[test]
    fn distinct_elements() {
        let ref mut vec: Vec<_> = (1..=10).collect();
        vec.shuffle(&mut thread_rng());
        let result = vec.largest();
        assert_eq!(result, Some(&mut 10));
        *result.unwrap() = 0;
        assert_eq!(vec.largest(), Some(&mut 9));
    }

    #[test]
    fn repeating_elements() {
        let ref mut vec: Vec<_> = vec![5, 1, 2, 7, 5, 6, 7, 2, 7, 6];
        let result = vec.largest();
        assert_eq!(result, Some(&mut 7));
    }
}
