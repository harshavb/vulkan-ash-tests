mod hello_triangle;

use hello_triangle::TriangleApplication;

fn main() {
    match TriangleApplication::run() {
        Ok(string) => println!("{}", string),
        Err(error) => println!("{}", error),
    }

    println!("test");
}
