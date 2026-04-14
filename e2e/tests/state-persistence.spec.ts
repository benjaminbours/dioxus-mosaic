import { test, expect } from "@playwright/test";
import { openApp, getHeaderCenter, getPanelBox, simulateDrag } from "./helpers";

test.describe("State Persistence", () => {
  test.beforeEach(async ({ page }) => {
    await openApp(page);
  });

  test("counter value survives after dragging the Interactive panel to a new location", async ({
    page,
  }) => {
    // Find the increment button inside the Counter panel and click it 3 times
    const counterPane = page.locator(".mosaic-tile-pane").filter({
      has: page.locator(".mosaic-tile-header", { hasText: "Interactive" }),
    });
    const incrementBtn = counterPane.locator("button", {
      hasText: "Increment",
    });

    await incrementBtn.click();
    await incrementBtn.click();
    await incrementBtn.click();

    // Verify counter reads 3
    await expect(counterPane.locator("strong")).toHaveText("3");

    // Drag Counter panel to the bottom of the Terminal panel
    const start = await getHeaderCenter(page, "Interactive");
    const terminalPane = await getPanelBox(page, "Terminal");
    const endX = terminalPane.x + terminalPane.width * 0.5;
    const endY = terminalPane.y + terminalPane.height * 0.9;

    await simulateDrag(page, start.x, start.y, endX, endY);
    await page.waitForTimeout(2000);

    // Re-locate the Counter panel after the layout change
    const counterPaneAfter = page.locator(".mosaic-tile-pane").filter({
      has: page.locator(".mosaic-tile-header", { hasText: "Interactive" }),
    });

    // The counter value should still be 3
    await expect(counterPaneAfter.locator("strong")).toHaveText("3");
  });
});
