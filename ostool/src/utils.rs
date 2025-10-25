pub trait ShellRunner{
    fn run(&self) -> anyhow::Result<()>;
}