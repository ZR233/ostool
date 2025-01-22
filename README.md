# ostool

Rust开发OS的工具集

## 使用

```shell
cargo install ostool
ostool --help
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
$remoteFile = "$remotePath/examples/helloworld/helloworld_aarch64-phytium-pi.elf"
$localTargetFile = "./target/kernel_raw.elf"

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

```toml
[compile]
target = "aarch64-unknown-none-softfloat"

[compile.custom]
shell = [
    [
        "pwsh -f ./remote_build.ps1",
    ]
]
elf = "target/kernel_raw.elf"

[qemu]
machine = "virt"
cpu = "cortex-a53"
graphic = false
args = "-smp 2"

[uboot]
serial = "COM3"
baud_rate = 115200
net = "以太网"
dtb_file = "tools\\phytium_pi\\phytiumpi_firefly.dtb"
```
