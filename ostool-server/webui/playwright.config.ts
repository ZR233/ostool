import { defineConfig } from "playwright/test";

const executablePath = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE;

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  use: {
    baseURL: "http://127.0.0.1:4174",
    browserName: "chromium",
    launchOptions: executablePath ? { executablePath } : undefined,
  },
  webServer: {
    command: "pnpm dev --host 127.0.0.1 --port 4174",
    url: "http://127.0.0.1:4174/admin/",
    reuseExistingServer: !process.env.CI,
  },
});
