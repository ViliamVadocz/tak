#[derive(Default)]
pub struct Edge {
    pub q_value: f32, // Q(s_t, a)
    pub policy: f32,  // P(s_t, a)
    pub visits: u32,  // N(s_t, a)
    pub virtual_visits: u32,
}

impl Edge {
    pub fn total_visits(&self) -> u32 {
        self.visits + self.virtual_visits
    }
}
