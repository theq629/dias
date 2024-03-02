pub struct DummyExiter;

impl super::Exiter for DummyExiter {
    fn exit(&mut self) {}
}
