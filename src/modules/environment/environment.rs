use super::{fog::Fog, sky::Sky, sun::Sun};

pub struct Environment {
    pub fog: Fog,
    pub sky: Sky,
    pub sun: Sun,
}