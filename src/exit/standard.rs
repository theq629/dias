pub struct Exiter;

impl Exiter {
    pub fn new() -> Self {
        Self
    }
}

impl super::Exiter for Exiter {
    fn exit(&mut self) {
        std::process::exit(0);
    }
}
