/// World-space alignment guides while dragging nodes (vertical x, horizontal y).
#[derive(Debug, Clone, Default)]
pub struct AlignmentGuides {
    pub vertical_x: Vec<f32>,
    pub horizontal_y: Vec<f32>,
}
