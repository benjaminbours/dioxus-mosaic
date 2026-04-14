import { test, expect } from "@playwright/test";
import { openApp, getHeaderCenter, getPanelBox, getPanelBoxMap, simulateDrag } from "./helpers";

test.describe("Layout Orientation Change", () => {
  test.beforeEach(async ({ page }) => {
    await openApp(page);
  });

  test("dragging Terminal to bottom of Files creates a vertical stack with ~50/50 split", async ({
    page,
  }) => {
    const start = await getHeaderCenter(page, "Terminal");
    const filesPane = await getPanelBox(page, "Files");
    const endX = filesPane.x + filesPane.width * 0.5;
    const endY = filesPane.y + filesPane.height * 0.9;

    await simulateDrag(page, start.x, start.y, endX, endY);
    await page.waitForTimeout(2000);

    const after = await getPanelBoxMap(page);
    const files = after["Files"];
    const terminal = after["Terminal"];

    expect(files).toBeDefined();
    expect(terminal).toBeDefined();

    const xOverlap = Math.max(
      0,
      Math.min(files.x + files.width, terminal.x + terminal.width) - Math.max(files.x, terminal.x),
    );
    expect(xOverlap).toBeGreaterThan(files.width * 0.5);

    const allBoxes = Object.values(after);
    const minX = Math.min(...allBoxes.map((b) => b.x));
    const maxX = Math.max(...allBoxes.map((b) => b.x + b.width));
    const totalWidth = maxX - minX;
    const leftRatio = files.width / totalWidth;

    expect(leftRatio).toBeGreaterThanOrEqual(0.35);
    expect(leftRatio).toBeLessThanOrEqual(0.65);
  });
});
