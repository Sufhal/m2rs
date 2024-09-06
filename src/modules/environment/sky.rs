use super::Color;

pub struct Sky {
    gradient: Gradient,
    day: Gradient,
    night: Gradient,
}

struct Gradient {
    colors: [Color; 5]
}