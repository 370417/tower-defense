pub struct Tower {
    pub row: usize,
    pub col: usize,
    pub range: Range,
}

#[derive(Clone, Copy)]
pub enum Range {
    Circle { radius: f32 },
}
