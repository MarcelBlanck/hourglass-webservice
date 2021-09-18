#[derive(Debug)]
pub struct HourglassState<T, U, V> {
    pub ticking: T,
    pub remaining_seconds: U,
    pub finalize: V
}

impl<T, U, V> HourglassState<T, U, V> {
    pub fn new(ticking: T, remaining_seconds: U, finalize: V) -> HourglassState<T, U, V> {
        HourglassState { ticking, remaining_seconds, finalize }
    }
}
