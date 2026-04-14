import { defineConfig, devices } from "@playwright/test";

const PORT = 9999;
const BASE_URL = `http://localhost:${PORT}`;

export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 3,
  reporter: process.env.CI ? "github" : "html",

  timeout: 60_000,

  use: {
    baseURL: BASE_URL,
    viewport: { width: 1280, height: 800 },
    navigationTimeout: 30_000,
    actionTimeout: 10_000,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  webServer: {
    command: `dx serve --example basic --platform web --port ${PORT} --open false --interactive false --watch false`,
    cwd: "..",
    url: BASE_URL,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
    stdout: "pipe",
    stderr: "pipe",
  },
});
