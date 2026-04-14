import { test, expect } from "@playwright/test";
import { openApp, simulateDragHoverOnly } from "./helpers";

test.describe("Drop Zone Highlights", () => {
  test.beforeEach(async ({ page }) => {
    await openApp(page);
  });

  test("visible drop zones appear when dragging over a panel", async ({ page }) => {
    await simulateDragHoverOnly(page, "Files", "Editor", 0.5, 0.15);

    const hasVisibleDropZone = await page.evaluate(() =>
      Array.from(document.querySelectorAll(".drop-zone")).some(
        (el) => getComputedStyle(el).display !== "none" && getComputedStyle(el).opacity !== "0",
      ),
    );
    expect(hasVisibleDropZone).toBe(true);
  });

  test("Editor panel shows 4 directional drop zones during drag", async ({ page }) => {
    await simulateDragHoverOnly(page, "Files", "Editor", 0.5, 0.15);

    const zoneCount = await page.evaluate(() => {
      const panes = Array.from(document.querySelectorAll(".mosaic-tile-pane"));
      const editorPane = panes.find((p) => {
        const h = p.querySelector(".mosaic-tile-header");
        return (h?.textContent || "").replace(/✕/g, "").trim() === "Editor";
      });
      if (!editorPane) return 0;
      return editorPane.querySelectorAll(".drop-zone").length;
    });
    expect(zoneCount).toBe(4);
  });
});
