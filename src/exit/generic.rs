pub trait Exiter {
    fn exit(&mut self);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn boxability() {
        // It is useful to be able to store exiters, which for practical purposes requires boxing.
        // So test that boxing is possible and not difficult.
        fn _test<E: 'static + Exiter>(exiter: E) {
            let _: Box<_> = Box::new(exiter);
        }
    }
}
