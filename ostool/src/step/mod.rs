use crate::project::Project;
use anyhow::Result;

mod compile;
mod prepare_test;
mod qemu;
mod tftp;
mod uboot;

pub use compile::*;
pub use prepare_test::*;
pub use qemu::*;
pub use tftp::*;
pub use uboot::*;

pub trait Step {
    fn run(&mut self, project: &mut Project) -> Result<()>;
}
