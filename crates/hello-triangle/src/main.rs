mod hello_triangle;

use hello_triangle::TriangleApplication;

fn main() {
    let (app, io) = match TriangleApplication::new() {
        Ok((app, io)) => (app, io),
        Err(error) => {
            println!("{}", error);
            panic!("got an error");
        }
    };
    hello_triangle::run(app, io);
}
