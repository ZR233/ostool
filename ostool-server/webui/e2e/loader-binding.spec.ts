import { expect, test } from "playwright/test";

const loaderDevice = {
  mac_address: "02:00:00:00:00:42",
  current_mac_address: "02:00:00:00:00:42",
  ip_address: "10.77.0.142",
  arch: "x86_64",
  loader_version: "0.2.0",
  hardware: {
    manufacturer: "QEMU",
    product: "Standard PC (Q35)",
    version: "pc-q35",
    serial: "virtual-42",
  },
  last_seen_at: "2026-09-04T00:00:00Z",
  online: true,
  conflict: false,
  bound_board_id: null,
  current_registration_id: "registration-42",
};

test("an unbound loader only pre-fills its MAC and reports a duplicate binding", async ({ page }) => {
  await page.route("**/api/v1/admin/boards", async (route) => {
    if (route.request().method() === "POST") {
      await route.fulfill({
        status: 409,
        contentType: "application/json",
        body: JSON.stringify({
          code: "mac_already_bound",
          message: "MAC 地址已绑定到另一块开发板",
        }),
      });
      return;
    }
    await route.fulfill({ json: [] });
  });
  await page.route("**/api/v1/admin/sessions", (route) =>
    route.fulfill({ json: { sessions: [] } }),
  );
  await page.route("**/api/v1/admin/loader-devices", (route) =>
    route.fulfill({ json: [loaderDevice] }),
  );
  await page.route("**/api/v1/admin/virtual-devices", (route) =>
    route.fulfill({ json: { enabled: false, devices: [] } }),
  );
  await page.route("**/api/v1/admin/serial-ports", (route) => route.fulfill({ json: [] }));
  await page.route("**/api/v1/admin/dtbs", (route) => route.fulfill({ json: [] }));
  await page.route("**/api/v1/admin/tftp/status", (route) =>
    route.fulfill({
      json: {
        status: {
          resolved_server_ip: "10.77.0.1",
          resolved_netmask: "255.255.255.0",
        },
      },
    }),
  );

  await page.goto("/admin/boards");
  await expect(page.getByText("Standard PC (Q35)")).toBeVisible();
  await expect(page.getByText("virtual-42")).toBeVisible();
  await page.getByRole("link", { name: "创建配置" }).click();

  await expect(page.getByLabel("板卡 MAC")).toHaveValue(loaderDevice.mac_address);
  await expect(page.getByLabel("板型", { exact: true })).toHaveValue("");
  await expect(page.getByLabel("电源管理类型")).toHaveValue("custom");
  await expect(page.getByLabel("启用串口")).not.toBeChecked();

  await page.getByLabel("板型", { exact: true }).fill("qemu-x86_64");
  await page.getByLabel("开机命令").fill("true");
  await page.getByLabel("关机命令").fill("true");
  await page.getByRole("button", { name: "保存配置" }).click();
  await expect(page.getByText("MAC 地址已绑定到另一块开发板")).toBeVisible();
});
