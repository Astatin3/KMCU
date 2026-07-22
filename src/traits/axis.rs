pub trait Axis {
    fn step(&self, direction: bool)
    where
        Self: Sized;
}
