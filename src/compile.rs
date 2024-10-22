use byte_unit::Byte;

use crate::{project::Project, shell::Shell};

pub struct Compile {}

impl Compile {
    pub fn run(project: &mut Project, debug: bool) {
        let bin_name = project
            .config
            .compile
            .kernel_bin_name
            .clone()
            .unwrap_or("kernel.bin".to_string());

        let bin_path = project.output_dir(debug).join(bin_name);

        let log_level = format!("{:?}", project.config.compile.log_level);

        let all_features = project.package_all_features();

        let features_log_level = format!("log/release_max_level_{}", log_level.to_lowercase());

        let mut features = project.config.compile.features.join(" ");

        let mut args = vec![
            "build",
            "-p",
            &project.config.compile.package,
            "--target",
            &project.config.compile.target,
        ];

        if !features.is_empty() {
            args.push("--features");
            args.push(&features);
        }

        if !debug {
            args.push("--release");
        }

        let rust_flags = project.config.compile.rust_flags.clone()
            + "  -C link-arg=-no-pie -C link-arg=-znostart-stop-gc";

        let mut cmd = project.shell("cargo");

        for (key, value) in &project.config.compile.env {
            cmd.env(key, value);
        }

        cmd.args(args).env("RUSTFLAGS", rust_flags).exec().unwrap();

        let elf = project
            .output_dir(debug)
            .join(&project.config.compile.package);

        let _ = std::fs::remove_file("target/kernel.elf");
        std::fs::copy(&elf, "target/kernel.elf").unwrap();

        project
            .shell("rust-objcopy")
            .args(["--strip-all", "-O", "binary"])
            .arg(&elf)
            .arg(&bin_path)
            .exec()
            .unwrap();

        let img_size = std::fs::metadata(&bin_path).unwrap().len();
        println!("kernel image size: {:#}", Byte::from_u64(img_size));

        project.bin_path = Some(bin_path)
    }
}
