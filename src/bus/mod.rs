//! Bus stuff

// Modules
pub mod memorybus;

use errors::*;

// Common things
pub trait BusDevice {
    /// Do whatever in a clock cycle.
    fn tick(&mut self) {}

    /// Initialize.
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
