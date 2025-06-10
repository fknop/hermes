pub trait Termination {
    fn should_terminate(&self) -> bool;
}
