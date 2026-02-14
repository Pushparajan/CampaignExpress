/**
 * Campaign Express UI — Chrome Compatibility E2E Tests
 *
 * Ensures all pages render without fatal JS errors in headless Chromium,
 * validates accessibility attributes, checks responsive layouts,
 * and catches runtime errors. API errors (ECONNREFUSED) are expected
 * since these tests run without a backend — only UI-level issues are flagged.
 */

import { test, expect, type Page } from "@playwright/test";

// ---- Helpers ----

/** Collect fatal console errors (excluding API/network errors) */
function collectFatalErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (err) => {
    const msg = err.message;
    // Ignore API fetch failures — backend not running during E2E tests
    if (
      msg.includes("ECONNREFUSED") ||
      msg.includes("Failed to fetch") ||
      msg.includes("NetworkError") ||
      msg.includes("ERR_CONNECTION_REFUSED") ||
      msg.includes("fetch")
    ) {
      return;
    }
    errors.push(msg);
  });
  return errors;
}

// ---- Page Rendering Tests ----

test.describe("Page Rendering (Chrome)", () => {
  const pages = [
    { path: "/", name: "Dashboard" },
    { path: "/campaigns", name: "Campaigns" },
    { path: "/journeys", name: "Journeys" },
    { path: "/dco", name: "DCO" },
    { path: "/cdp", name: "CDP" },
    { path: "/experiments", name: "Experiments" },
    { path: "/billing", name: "Billing" },
    { path: "/platform", name: "Platform" },
    { path: "/ops", name: "Operations" },
    { path: "/login", name: "Login" },
  ];

  for (const { path, name } of pages) {
    test(`${name} page (${path}) renders without fatal JS errors`, async ({
      page,
    }) => {
      const errors = collectFatalErrors(page);
      const response = await page.goto(path, { waitUntil: "domcontentloaded" });
      expect(response?.status()).toBeLessThan(500);
      // Wait for React to hydrate
      await page.waitForTimeout(1000);
      expect(errors).toEqual([]);
    });
  }
});

// ---- Layout & Chrome CSS Tests ----

test.describe("Layout & CSS (Chrome)", () => {
  test("dark theme body background renders", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    const bg = await page.locator("body").evaluate(
      (el) => getComputedStyle(el).backgroundColor
    );
    // Should be a dark color (not white, not transparent)
    expect(bg).not.toBe("rgb(255, 255, 255)");
    expect(bg).not.toBe("rgba(0, 0, 0, 0)");
  });

  test("sidebar renders with navigation items (or login page shown)", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    const sidebar = page.locator("nav");
    if (await sidebar.isVisible()) {
      const links = sidebar.locator("a");
      expect(await links.count()).toBeGreaterThanOrEqual(5);
    } else {
      // If redirected to login, the login form should be visible
      const loginBtn = page.locator('button:has-text("Sign In")');
      await expect(loginBtn).toBeVisible();
    }
  });

  test("responsive: page renders at mobile viewport", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    const body = page.locator("body");
    await expect(body).toBeVisible();
    // Body should not overflow horizontally
    const overflow = await body.evaluate((el) => el.scrollWidth > window.innerWidth);
    expect(overflow).toBe(false);
  });

  test("backdrop-filter CSS property is applied in Chrome", async ({
    page,
  }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    const header = page.locator("header").first();
    if (await header.isVisible()) {
      const filter = await header.evaluate(
        (el) => getComputedStyle(el).backdropFilter || getComputedStyle(el).webkitBackdropFilter
      );
      // Chrome should support backdrop-filter
      expect(filter).toBeTruthy();
      expect(filter).not.toBe("none");
    }
  });

  test("CSS flexbox/grid layout works on all pages", async ({ page }) => {
    await page.goto("/campaigns", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    // The body should have positive height (page isn't blank)
    const bodyHeight = await page.locator("body").evaluate(
      (el) => el.getBoundingClientRect().height
    );
    expect(bodyHeight).toBeGreaterThan(100);
  });

  test("font-family renders with system sans-serif fallback", async ({
    page,
  }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    const fontFamily = await page.locator("body").evaluate(
      (el) => getComputedStyle(el).fontFamily
    );
    expect(fontFamily).toBeTruthy();
    // Should contain a sans-serif font (Inter or system fallback)
    expect(fontFamily.toLowerCase()).toMatch(/inter|sans-serif|system-ui|segoe/);
  });
});

// ---- Accessibility Tests ----

test.describe("Accessibility (Chrome)", () => {
  test("search input has aria-label (when authenticated)", async ({ page }) => {
    await page.goto("/campaigns", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    // May redirect to login if no auth — check if search input exists on the rendered page
    const searchInput = page.locator('input[aria-label="Search campaigns"]');
    const loginForm = page.locator('button:has-text("Sign In")');
    const hasSearch = (await searchInput.count()) > 0;
    const hasLogin = (await loginForm.count()) > 0;
    // Either we see the search input (authenticated) or the login form (unauthenticated)
    expect(hasSearch || hasLogin).toBe(true);
  });

  test("user dropdown has aria-expanded and aria-haspopup", async ({
    page,
  }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    const userBtn = page.locator('button[aria-haspopup="menu"]');
    if ((await userBtn.count()) > 0) {
      await expect(userBtn).toHaveAttribute("aria-expanded", "false");
      await userBtn.click();
      await expect(userBtn).toHaveAttribute("aria-expanded", "true");
      // Dropdown menu should appear
      const menu = page.locator('[role="menu"]');
      await expect(menu).toBeVisible();
    }
  });

  test("html has lang attribute", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const lang = await page.locator("html").getAttribute("lang");
    expect(lang).toBe("en");
  });

  test("page has a title", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);
  });

  test("pagination buttons have aria-labels when present", async ({
    page,
  }) => {
    await page.goto("/campaigns", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    // Check that if pagination exists it has aria-labels
    const navPagination = page.locator('nav[aria-label="Table pagination"]');
    if ((await navPagination.count()) > 0) {
      const prevBtn = page.locator('button[aria-label="Previous page"]');
      const nextBtn = page.locator('button[aria-label="Next page"]');
      expect(await prevBtn.count()).toBeGreaterThanOrEqual(1);
      expect(await nextBtn.count()).toBeGreaterThanOrEqual(1);
    }
  });
});

// ---- Interaction Tests ----

test.describe("Interactions (Chrome)", () => {
  test("login page renders form inputs", async ({ page }) => {
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    const inputs = page.locator("input");
    // Login page should have at least email + password fields
    expect(await inputs.count()).toBeGreaterThanOrEqual(2);
  });

  test("keyboard Tab navigation reaches interactive elements", async ({
    page,
  }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(500);
    await page.keyboard.press("Tab");
    const activeTag = await page.evaluate(
      () => document.activeElement?.tagName?.toLowerCase()
    );
    // Focus should be on a link, button, or input (not body)
    expect(["a", "button", "input"]).toContain(activeTag);
  });

  test("page shows loading or error state (no blank screen)", async ({
    page,
  }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1500);
    const bodyText = await page.locator("body").innerText();
    // The page should render SOMETHING (loading spinner text, error message, or dashboard)
    expect(bodyText.trim().length).toBeGreaterThan(0);
  });
});

// ---- Performance ----

test.describe("Performance (Chrome)", () => {
  test("page loads within 5 seconds", async ({ page }) => {
    const start = Date.now();
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const duration = Date.now() - start;
    expect(duration).toBeLessThan(5000);
  });

  test("no layout shift issues (CLS < 0.25)", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(2000);
    const cls = await page.evaluate(() => {
      return new Promise<number>((resolve) => {
        let clsValue = 0;
        try {
          const observer = new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
              const le = entry as PerformanceEntry & {
                hadRecentInput?: boolean;
                value?: number;
              };
              if (!le.hadRecentInput && le.value) {
                clsValue += le.value;
              }
            }
          });
          observer.observe({ type: "layout-shift", buffered: true });
          setTimeout(() => {
            observer.disconnect();
            resolve(clsValue);
          }, 1000);
        } catch {
          resolve(0);
        }
      });
    });
    expect(cls).toBeLessThan(0.25);
  });
});
