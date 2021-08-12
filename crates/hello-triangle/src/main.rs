mod hello_triangle;

use hello_triangle::TriangleApplication;

fn main() {
    let (app, io) = TriangleApplication::new();
    hello_triangle::run(app, io);
}