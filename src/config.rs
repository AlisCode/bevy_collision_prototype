pub struct BevyCollisionConfig {
    pub scale: f32,
}

impl Default for BevyCollisionConfig {
    fn default() -> Self {
        BevyCollisionConfig { scale: 1. }
    }
}
