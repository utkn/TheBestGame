#[derive(Clone, Copy, Debug)]
pub struct AiController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fraction {
    GoodGuy,
    BadGuy,
    AntiHero,
}
