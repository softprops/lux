use super::rand;
use super::term;

pub struct Wheel {
    colors: Vec<term::color::Color>,
    rng: rand::ThreadRng,
}

impl Wheel {
    pub fn new() -> Wheel {
        Wheel {
            colors: vec![term::color::CYAN,
                         term::color::MAGENTA,
                         term::color::GREEN,
                         term::color::YELLOW,
                         term::color::BRIGHT_BLUE],
            rng: rand::thread_rng(),
        }
    }
}

impl Iterator for Wheel {
    type Item = term::color::Color;
    fn next(&mut self) -> Option<term::color::Color> {
        Some(rand::sample(&mut self.rng, self.colors.clone(), 1)[0])
    }
}
