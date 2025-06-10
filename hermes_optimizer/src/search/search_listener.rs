pub trait SearchListener {
    fn search_start(&mut self) {}
    fn iteration_start(&mut self) {}
    fn iteration_end(&mut self) {}
    fn search_end(&mut self) {}
}
