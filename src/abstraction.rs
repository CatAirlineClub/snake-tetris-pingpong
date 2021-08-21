
#[derive(Debug, PartialEq, Eq)]
enum Direction {
    Up(Up), Down(Down), Left(Left), Right(Right), JumpRight(JumpRight)
}

trait Vector {
    fn direction() -> Option<Direction>;
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)] pub struct Up;
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)] pub struct Down;
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)] pub struct Left;
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)] pub struct Right;
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)] pub struct JumpRight;

impl Vector for Up {
    fn direction() -> Option<Direction> { Some(Direction::Up(Up)) }
}

impl Vector for Down {
    fn direction() -> Option<Direction> { Some(Direction::Down(Down)) }
}

impl Vector for Left {
    fn direction() -> Option<Direction> { Some(Direction::Left(Left)) }
}

impl Vector for Right {
    fn direction() -> Option<Direction> { Some(Direction::Right(Right)) }
}

impl Vector for JumpRight {
    fn direction() -> Option<Direction> { Some(Direction::JumpRight(JumpRight)) }
}

impl Vector for () {
    fn direction() -> Option<Direction> { None }
}

trait Snake {
    type Next: Vector;
    fn next() -> Option<Self::Next>;
    fn direction() -> Direction;
}

#[derive(Clone, Copy, Debug)]
pub struct Flow<Direction, Tail>(Direction, Tail);

impl<Head, Tail> Default for Flow<Head, Tail> where Head: Default, Tail: Default {
    fn default() -> Self {
        Flow(Default::default(), Default::default())
    }
}

#[derive(Debug)]
pub enum Error {
    Collision
}

pub trait Render {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error>;
}

impl Render for Flow<(), ()> {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {
        Ok((grid, pos))
    }
}

impl Render for () {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {
        Ok((grid, pos))
    }
}

impl<Tail: Render> Render for Flow<Up, Tail> {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {  
        step(self.1, grid, pos,
         |pos| pos.0 == 0,
         |mut pos| { pos.0 -= 1; pos })
    }
}

impl<Tail: Render> Render for Flow<Down, Tail> {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> { 
        step(self.1, grid, pos,
         |pos| pos.0 == crate::W - 1,
         |mut pos| { pos.0 += 1; pos })
    }
}

impl<Tail: Render>  Render for Flow<Left, Tail> {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {
        step(self.1, grid, pos,
         |pos| pos.1 == 0,
         |mut pos| { pos.1 -= 1; pos })
    }
}

impl<Tail: Render>  Render for Flow<Right, Tail> {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {
        step(self.1, grid, pos,
            |pos| pos.1 == crate::H - 1,
            |mut pos| { pos.1 += 1; pos })
    }
}

impl Render for Flow<JumpRight, ()> {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {
        step(self.1, grid, pos,
            |pos| pos.1 >= crate::H - 2,
            |mut pos| { pos.1 += 2; pos })
    }
}

impl<Head, Tail> Render for Flow<Direction, Flow<Head, Tail>> where Flow<Head, Tail>: Render {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {
        let (grid, pos) = match self.0 {
            Direction::Up(_) => Flow(Up, ()).render(grid, pos),
            Direction::Down(_) => Flow(Down, ()).render(grid, pos),
            Direction::Left(_) => Flow(Left, ()).render(grid, pos),
            Direction::Right(_) => Flow(Right, ()).render(grid, pos),
            Direction::JumpRight(_) => Flow(JumpRight, ()).render(grid, pos),
        }?;

        self.1.render(grid, pos)
    }
}

fn step<Tail: Render>(
    tail: Tail,
    mut grid: Grid,
    pos: Position,
    check: impl Fn(Position) -> bool,
    mutation: impl Fn(Position)->Position) -> Result<(Grid, Position), Error> {

    if check(pos) {
        return Err(Error::Collision)
    }
    let pos = mutation(pos);
    if grid.pos(pos).present() {
        return Err(Error::Collision)
    }
    grid.set(pos);

    tail.render(grid, pos)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Present, Void
}

impl Form {
    pub fn present(&self) -> bool {
        self == &Self::Present
    }
}

#[derive(Debug)]
pub struct Grid([[Form; crate::W]; crate::H]);

impl Grid {
    pub fn pos(&self, pos: Position) -> Form {
        self.0[pos.0][pos.1]
    }
    fn set(&mut self, pos: Position) {
        self.0[pos.0][pos.1] = Form::Present
    }
}

impl Default for Grid {
    fn default() -> Self {
        Grid([[Form::Void; crate::W]; crate::H])
    }
}

#[derive(Clone, Copy)]
pub struct Position(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub enum Piece {
    Square(Flow<Up, Flow<Right, Flow<Down, ()>>>),
    Line(Flow<Down, Flow<Down, Flow<Down, ()>>>),
    LeftTwist(Flow<Down, Flow<Right, Flow<Down, ()>>>),
    RightTwist(Flow<Down, Flow<Left, Flow<Down, ()>>>),
    Pile(Flow<Down, Flow<Left, Flow<JumpRight, ()>>>),
}

impl Render for Piece {
    fn render(self, grid: Grid, pos: Position) -> Result<(Grid, Position), Error> {
        match self {
            Piece::Square(x) => x.render(grid, pos),
            Piece::Line(x) => x.render(grid, pos),
            Piece::LeftTwist(x) => x.render(grid, pos),
            Piece::RightTwist(x) => x.render(grid, pos),
            Piece::Pile(x) => x.render(grid, pos),
        }
    }
}

impl Piece {
    pub fn random() -> Self {
        rand::random()
    }
}

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

impl Distribution<Piece> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Piece {
        // match rng.gen_range(0, 3) { // rand 0.5, 0.6, 0.7
        match rng.gen_range(0..5) { // rand 0.8
            0 => Piece::Square(Default::default()),
            1 => Piece::Line(Default::default()),
            2 => Piece::LeftTwist(Default::default()),
            3 => Piece::RightTwist(Default::default()),
            _ => Piece::Pile(Default::default()),
        }
    }
}
