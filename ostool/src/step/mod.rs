use crate::project::Project;
use anyhow::Result;

mod compile;
mod qemu;
mod test_prepare;
mod tftp;
mod uboot;

pub use compile::*;
pub use qemu::*;
pub use test_prepare::*;
pub use tftp::*;
pub use uboot::*;

pub trait Step {
    fn run(&mut self, project: &mut Project) -> Result<()>;
}
