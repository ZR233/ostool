# ostool

Rust开发OS的工具集，可以方便的通过 Qemu 和 U-Boot 启动。

## 使用

```shell
cargo install ostool
ostool --help
```

### 配置文件

ArceOS为例

进入工作目录

```shell
# 生成默认配置文件
ostool defconfig
```

`.project.toml`: 

```toml
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
# 编译命令，可多条
shell = [["make ARCH=aarch64 A=examples/helloworld FEATURES=page-alloc-4g"]]
# 要启动的内核
kernel = "examples/helloworld/helloworld_aarch64-qemu-virt.bin"

[qemu]
machine = "virt"
cpu = "cortex-a57"
graphic = false
args = ""
```

### Qemu启动

```shell
ostool run qemu
# debug
ostool run qemu -d
```

### U-Boot 启动

linux tftp 使用69端口，为特权接口，需要为应用授予权限：

```shell
sudo setcap cap_net_bind_service=+eip $(which ostool)
```

```shell
ostool run uboot
```

### 远程构建示例

```pwsh
# remote_build.ps1

# 定义远程服务器的连接信息
$remoteHost = "{ip}"
$username = "{name}"
$remotePath = "/home/arceos/"
$makeCommand = "make A=examples/helloworld PLATFORM=aarch64-phytium-pi "
$remoteFile = "$remotePath/examples/helloworld/helloworld_aarch64-phytium-pi.bin"
$localTargetFile = "./target/kernel_raw.bin"

# 使用 SSH 连接到远程服务器并执行命令
ssh "$username@$remoteHost" "cd $remotePath;. ~/.profile;$makeCommand"

if ($?) {
    Write-Host "remote build ok"

}
else {
    Write-Host "remote build fail"
    exit 1
}

# 使用 SCP 将远程文件拷贝到本地目标路径并重命名为 kernel.elf
$cmd = "scp $username@${remoteHost}:${remoteFile} $localTargetFile"
Write-Host "exec: $cmd"
Invoke-Expression $cmd
if ($?) {
    Write-Host "copy ok"
}
else {
    Write-Host "copy fail"
    exit 1
}
```

`.project.toml`

```toml
[compile]
target = "aarch64-unknown-none-softfloat"

[compile.build.Custom]
shell = [
    [
        "pwsh -f ./remote_build.ps1",
    ]
]
kernel = "target/kernel_raw.bin"

[qemu]
machine = "virt"
cpu = "cortex-a53"
graphic = false
args = "-smp 2"

[uboot]
serial = "COM3"
baud_rate = 115200
dtb_file = "tools/phytium_pi/phytiumpi_firefly.dtb"
```
