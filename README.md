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
