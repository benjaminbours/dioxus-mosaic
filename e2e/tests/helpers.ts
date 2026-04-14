import { type Page } from "@playwright/test";

/**
 * Drag simulation strategy
 *
 * Assumption: Playwright's high-level `page.dragAndDrop()` dispatches
 * synthetic JS drag events (dragstart/dragover/drop). Dioxus's WASM event
 * system listens for mousedown/mousemove/mouseup — not the HTML drag-and-drop
 * API — so the high-level helper does nothing useful here.
 *
 * Instead, we use `page.mouse` which sends trusted, browser-level mouse events
 * over Playwright's CDP connection that Dioxus picks up reliably.
 */

export interface PanelBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export async function openApp(page: Page): Promise<void> {
  await page.goto("/", { waitUntil: "commit", timeout: 60_000 });

  const appeared = await page
    .waitForSelector(".mosaic-tile-pane", { timeout: 10_000 })
    .then(() => true)
    .catch(() => false);

  if (!appeared) {
    await page.reload({ waitUntil: "domcontentloaded", timeout: 60_000 });
    await page.waitForSelector(".mosaic-tile-pane", { timeout: 60_000 });
  }

  await page.evaluate(() => document.querySelector(".dx-toast")?.remove());
}

export async function getPanelCount(page: Page): Promise<number> {
  return page.locator(".mosaic-tile-pane").count();
}

export async function getPanelTitles(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    Array.from(document.querySelectorAll(".mosaic-tile-header")).map((el) =>
      (el.textContent || "").replace(/✕/g, "").trim(),
    ),
  );
}

export async function getPanelBoxMap(page: Page): Promise<Record<string, PanelBox>> {
  return page.evaluate(() => {
    const titles = Array.from(document.querySelectorAll(".mosaic-tile-header")).map((el) =>
      (el.textContent || "").replace(/✕/g, "").trim(),
    );

    const panes = Array.from(document.querySelectorAll(".mosaic-tile-pane"));
    const result: Record<string, { x: number; y: number; width: number; height: number }> = {};

    for (let i = 0; i < titles.length && i < panes.length; i++) {
      const r = panes[i].getBoundingClientRect();
      result[titles[i]] = {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
      };
    }
    return result;
  });
}

export async function getHeaderCenter(
  page: Page,
  title: string,
): Promise<{ x: number; y: number }> {
  const pos = await page.evaluate((t) => {
    const headers = Array.from(document.querySelectorAll(".mosaic-tile-header"));
    const header = headers.find((h) => (h.textContent || "").replace(/✕/g, "").trim() === t);
    if (!header) return null;
    const r = header.getBoundingClientRect();
    return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
  }, title);

  if (!pos) throw new Error(`Header not found: ${title}`);
  return pos;
}

export async function getPanelBox(page: Page, title: string): Promise<PanelBox> {
  const box = await page.evaluate((t) => {
    const panes = Array.from(document.querySelectorAll(".mosaic-tile-pane"));
    const pane = panes.find((p) => {
      const h = p.querySelector(".mosaic-tile-header");
      return (h?.textContent || "").replace(/✕/g, "").trim() === t;
    });
    if (!pane) return null;
    const r = pane.getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height };
  }, title);

  if (!box) throw new Error(`Panel not found: ${title}`);
  return box;
}

export async function simulateDrag(
  page: Page,
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  steps = 14,
): Promise<void> {
  const n = Math.max(steps, 1);

  await page.mouse.move(startX, startY);
  await page.waitForTimeout(50);
  await page.mouse.down();
  await page.waitForTimeout(100);

  for (let i = 1; i <= n; i++) {
    const t = i / n;
    const x = startX + (endX - startX) * t;
    const y = startY + (endY - startY) * t;
    await page.mouse.move(x, y);
    await page.waitForTimeout(30);
  }

  await page.waitForTimeout(200);
  await page.mouse.up();
}

export async function simulateDragHoverOnly(
  page: Page,
  sourceTitle: string,
  targetTitle: string,
  targetRelX: number,
  targetRelY: number,
  steps = 12,
): Promise<void> {
  const start = await getHeaderCenter(page, sourceTitle);
  const target = await getPanelBox(page, targetTitle);
  const endX = target.x + target.width * targetRelX;
  const endY = target.y + target.height * targetRelY;

  const n = Math.max(steps, 1);

  await page.mouse.move(start.x, start.y);
  await page.waitForTimeout(50);
  await page.mouse.down();
  await page.waitForTimeout(100);

  for (let i = 1; i <= n; i++) {
    const t = i / n;
    const x = start.x + (endX - start.x) * t;
    const y = start.y + (endY - start.y) * t;
    await page.mouse.move(x, y);
    await page.waitForTimeout(30);
  }

  await page.waitForTimeout(200);
}
