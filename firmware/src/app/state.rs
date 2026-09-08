#[derive(Clone, Copy, Default)]
pub struct AppState {
    pub rotation_counts: i64,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
