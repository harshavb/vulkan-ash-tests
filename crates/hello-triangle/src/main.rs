mod hello_triangle;

use hello_triangle::TriangleApplication;

fn main() {
    let (app, io) = TriangleApplication::new().unwrap();
    hello_triangle::run(app, io);
}
