use std::error::Error;

#[derive(Debug)]
pub enum AvailabilityError {
    NotSupported,
    NotAvailable(Option<Box<dyn Error>>),
}
