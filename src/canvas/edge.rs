use gpui::{Pixels, Point};

pub struct EdgeGeometry {
    pub start: Point<Pixels>,
    pub c1: Point<Pixels>,
    pub c2: Point<Pixels>,
    pub end: Point<Pixels>,
}
