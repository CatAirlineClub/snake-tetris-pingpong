
mod abstraction;

const W: usize = 20;
const H: usize = 20;

fn main() {
    for _ in 0 .. 15 {
        println!("{:?}", rand::random::<abstraction::Piece>())
    }
}
