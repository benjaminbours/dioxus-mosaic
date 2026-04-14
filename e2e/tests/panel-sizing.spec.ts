import { test, expect } from "@playwright/test";
import { openApp, getHeaderCenter, getPanelBox, getPanelBoxMap, simulateDrag } from "./helpers";

test.describe("Panel Sizing", () => {
  test.beforeEach(async ({ page }) => {
    await openApp(page);
  });

  test("dragging Files to Terminal right zone keeps reasonable panel dimensions", async ({
    page,
  }) => {
    const start = await getHeaderCenter(page, "Files");
    const terminalPane = await getPanelBox(page, "Terminal");
    const endX = terminalPane.x + terminalPane.width * 0.9;
    const endY = terminalPane.y + terminalPane.height * 0.5;

    await simulateDrag(page, start.x, start.y, endX, endY);
    await page.waitForTimeout(400);

    const boxes = await getPanelBoxMap(page);
    for (const [title, b] of Object.entries(boxes)) {
      expect(b.width, `panel "${title}" width too small`).toBeGreaterThanOrEqual(50);
      expect(b.height, `panel "${title}" height too small`).toBeGreaterThanOrEqual(50);
    }
  });

  test("Files and Terminal have approximately 50/50 horizontal split after drop", async ({
    page,
  }) => {
    const start = await getHeaderCenter(page, "Files");
    const terminalPane = await getPanelBox(page, "Terminal");
    const endX = terminalPane.x + terminalPane.width * 0.9;
    const endY = terminalPane.y + terminalPane.height * 0.5;

    await simulateDrag(page, start.x, start.y, endX, endY);
    await page.waitForTimeout(400);

    const boxes = await getPanelBoxMap(page);
    const files = boxes["Files"];
    const terminal = boxes["Terminal"];

    expect(files).toBeDefined();
    expect(terminal).toBeDefined();

    const unionLeft = Math.min(files.x, terminal.x);
    const unionRight = Math.max(files.x + files.width, terminal.x + terminal.width);
    const unionWidth = unionRight - unionLeft;
    const filesRatio = files.width / unionWidth;

    expect(filesRatio).toBeGreaterThanOrEqual(0.35);
    expect(filesRatio).toBeLessThanOrEqual(0.65);
  });

  test("Files and Terminal heights are balanced after drop", async ({ page }) => {
    const start = await getHeaderCenter(page, "Files");
    const terminalPane = await getPanelBox(page, "Terminal");
    const endX = terminalPane.x + terminalPane.width * 0.9;
    const endY = terminalPane.y + terminalPane.height * 0.5;

    await simulateDrag(page, start.x, start.y, endX, endY);
    await page.waitForTimeout(400);

    const boxes = await getPanelBoxMap(page);
    const files = boxes["Files"];
    const terminal = boxes["Terminal"];

    expect(files).toBeDefined();
    expect(terminal).toBeDefined();

    const heightFillRatio =
      Math.min(files.height, terminal.height) / Math.max(files.height, terminal.height);
    expect(heightFillRatio).toBeGreaterThanOrEqual(0.6);
  });
});
