pub(crate) struct Hours(pub(crate) f32);

impl From<f32> for Hours {
    fn from(hours: f32) -> Self {
        Self(hours)
    }
}
