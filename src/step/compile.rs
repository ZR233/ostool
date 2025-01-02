use byte_unit::Byte;

use crate::{project::Project, shell::Shell, step::Step};

pub struct Compile {
    is_debug: bool,
}

impl Compile {
    pub fn new_boxed(is_debug: bool) -> Box<dyn Step> {
        Box::new(Self { is_debug })
    }
}

impl Step for Compile {
    fn run(&mut self, project: &mut Project) -> anyhow::Result<()> {
        let bin_name = project
            .config_ref()
            .compile
            .kernel_bin_name
            .clone()
            .unwrap_or("kernel.bin".to_string());

        project.out_dir = Some(project.out_dir_with_profile(self.is_debug));

        let bin_path = project.out_dir().join(bin_name);

        let log_level = format!("{:?}", project.config_ref().compile.log_level);

        let mut features = project.config_ref().compile.features.join(" ");

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
            &project.config_ref().compile.package,
            "--target",
            &project.config_ref().compile.target,
            "-Z",
            "unstable-options",
        ];

        if !self.is_debug {
            args.push("--release");
        }

        let rust_flags = format!(
            "{} -Clink-args=-Map=target/kernel.map",
            project.config_ref().compile.rust_flags
        );

        let mut cmd = project.shell("cargo");

        for (key, value) in &project.config_ref().compile.env {
            cmd.env(key, value);
        }

        cmd.env("RUSTFLAGS", rust_flags).args(args);

        if !features.is_empty() {
            cmd.arg("--features");
            let features_str = features.split_whitespace().collect::<Vec<_>>().join(" ");
            cmd.arg(features_str);
        }
        cmd.exec(project.is_print_cmd).unwrap();

        let elf = project
            .out_dir()
            .join(&project.config_ref().compile.package);

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
}
