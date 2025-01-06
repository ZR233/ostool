use std::fs;

use byte_unit::Byte;

use crate::{
    config::compile::{CargoBuild, CustomBuild},
    project::Project,
    shell::Shell,
    step::Step,
};

pub struct Compile {
    is_debug: bool,
}

impl Compile {
    pub fn new_boxed(is_debug: bool) -> Box<dyn Step> {
        Box::new(Self { is_debug })
    }

    fn run_cargo(&mut self, project: &mut Project, config: CargoBuild) -> anyhow::Result<()> {
        let bin_name = config
            .kernel_bin_name
            .clone()
            .unwrap_or("kernel.bin".to_string());

        project.out_dir = Some(project.out_dir_with_profile(self.is_debug));

        let bin_path = project.out_dir().join(bin_name);

        let log_level = format!("{:?}", config.log_level);

        let mut features = config.features.join(" ");

        let deps = project.package_dependencies();
        if deps.contains(&"log".to_string()) {
            let features_log_level = format!("log/release_max_level_{}", log_level.to_lowercase());
            features += " ";
            features += &features_log_level;
        }

        let manifest_path = project.workdir().join("Cargo.toml");

        let mut args = vec![
            "build",
            "--manifest-path",
            &manifest_path.to_str().unwrap(),
            "-p",
            &config.package,
            "--target",
            &project.config_ref().compile.target,
            "-Z",
            "unstable-options",
        ];

        if !self.is_debug {
            args.push("--release");
        }

        let rust_flags = format!("{} -Clink-args=-Map=target/kernel.map", config.rust_flags);

        let mut cmd = project.shell("cargo");

        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        cmd.env("RUSTFLAGS", rust_flags).args(args);

        if !features.is_empty() {
            cmd.arg("--features");
            let features_str = features.split_whitespace().collect::<Vec<_>>().join(" ");
            cmd.arg(features_str);
        }
        cmd.exec(project.is_print_cmd).unwrap();

        let elf = project.out_dir().join(&config.package);

        project.elf_path = Some(elf.clone());

        let _ = std::fs::remove_file("target/kernel.elf");
        std::fs::copy(&elf, "target/kernel.elf").unwrap();

        project
            .shell("rust-objcopy")
            .args(["--strip-all", "-O", "binary"])
            .arg(&elf)
            .arg(&bin_path)
            .exec(project.is_print_cmd)
            .unwrap();

        let img_size = std::fs::metadata(&bin_path).unwrap().len();
        println!("kernel image size: {:#}", Byte::from_u64(img_size));

        project.set_binaries(elf, bin_path);

        Ok(())
    }

    fn run_custom(&self, project: &mut Project, config: CustomBuild) {
        for cmd in &config.shell {
            let mut cmd_iter = cmd.split_whitespace();

            let mut p = project.shell(cmd_iter.next().unwrap());

            for arg in cmd_iter {
                p.arg(arg.trim_matches('\"'));
            }

            p.exec(project.is_print_cmd).unwrap();
        }

        let _ = fs::create_dir("target");
        let _ = std::fs::remove_file("target/kernel.elf");

        let elf = config.elf.clone();

        std::fs::copy(&elf, "target/kernel.elf").unwrap();

        project.out_dir = Some(
            project
                .workspace_root()
                .join("target")
                .join(&project.config_ref().compile.target)
                .join("release"),
        );

        let _ = std::fs::create_dir_all(project.out_dir());

        let bin_path = project.out_dir().join("kernel.bin");

        project
            .shell("rust-objcopy")
            .args(["--strip-all", "-O", "binary"])
            .arg(&elf)
            .arg(&bin_path)
            .exec(project.is_print_cmd)
            .unwrap();

        let img_size = std::fs::metadata(&bin_path).unwrap().len();
        println!("kernel image size: {:#}", Byte::from_u64(img_size));

        project.set_binaries(elf.into(), bin_path);
    }
}

impl Step for Compile {
    fn run(&mut self, project: &mut Project) -> anyhow::Result<()> {
        if let Some(config) = project.config_ref().compile.cargo.clone() {
            self.run_cargo(project, config)?;

            return Ok(());
        }

        if let Some(config) = project.config_ref().compile.custom.clone() {
            self.run_custom(project, config);

            return Ok(());
        }

        panic!("配置文件错误，请删除 .project.yaml 文件后重试")
    }
}
