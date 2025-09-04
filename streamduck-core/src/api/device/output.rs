use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub position_size: Option<((i32, i32), (u32, u32))>,
    pub attached_to_input: Option<usize>,
    pub capabilities: Vec<OutputCapability>
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum OutputCapability {
    Display(u32, u32),
    Light,
    RGBLight
}