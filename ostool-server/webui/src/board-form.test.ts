import { describe, expect, it } from "vitest";
import {
  defaultFormState,
  boardToFormState,
  buildRequestPayload,
  validateForm,
  buildPowerManagementConfig,
} from "./board-form";
import type { BoardConfig } from "./types/api";
describe("board configuration contracts", () => {
  it("allows power configuration before board identity and HTTPboot MAC exist", () => {
    const f = defaultFormState();
    f.power_on_cmd = "on";
    f.power_off_cmd = "off";
    f.boot_kind = "httpboot";
    expect(buildPowerManagementConfig(f)).toEqual({
      kind: "custom",
      power_on_cmd: "on",
      power_off_cmd: "off",
    });
    expect(validateForm(f)).toContain("MAC");
  });
  it("round trips all existing U-Boot and power fields", () => {
    const b: BoardConfig = {
      id: "a",
      board_type: "arm",
      tags: ["x", "y"],
      disabled: true,
      notes: "note",
      serial: { key: { kind: "usb_path", value: "stable" }, baud_rate: 921600 },
      power_management: {
        kind: "zhongsheng_relay",
        key: { kind: "serial_number", value: "relay" },
      },
      network_identity: { mac_address: "02:00:00:00:00:01" },
      boot: {
        kind: "uboot",
        use_tftp: true,
        dtb_name: "one.dtb",
        kernel_load_addr: "0x1",
        fit_load_addr: "0x2",
        bootm_addr: "0x3",
        network_mode: "static_ip",
        board_ip: "10.0.0.2",
        server_ip: "10.0.0.1",
        netmask: "255.255.255.0",
        gatewayip: "10.0.0.1",
      },
    };
    expect(buildRequestPayload(boardToFormState(b))).toEqual({
      ...b,
      network_identity: null,
    });
  });
  it("preserves PXE notes and manually entered MAC", () => {
    const f = defaultFormState();
    f.board_type = "arm";
    f.power_on_cmd = "on";
    f.power_off_cmd = "off";
    f.boot_kind = "pxe";
    f.pxe_notes = "PXE notes";
    f.network_mac = "02:00:00:00:00:22";
    expect(buildRequestPayload(f).boot).toEqual({
      kind: "pxe",
      notes: "PXE notes",
    });
    expect(buildRequestPayload(f).network_identity?.mac_address).toBe(
      f.network_mac,
    );
  });
  it("rejects QEMU mismatched serial ownership and missing HTTPboot MAC", () => {
    const f = defaultFormState();
    f.power_management_kind = "qemu";
    f.virtual_device_id = "one";
    f.serial_enabled = true;
    f.serial_key_kind = "qemu";
    f.serial_key_value = "other";
    f.boot_kind = "httpboot";
    expect(validateForm(f)).toContain("同一个虚拟设备");
    expect(validateForm(f)).toContain("MAC");
  });
});
