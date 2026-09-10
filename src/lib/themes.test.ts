import { describe, expect, it } from "vitest";
import { DEFAULT_THEME_ID, themes } from "./themes";

describe("macOS default theme", () => {
  it("keeps the default theme id stable", () => {
    expect(DEFAULT_THEME_ID).toBe("github-dark");
    expect(themes["github-dark"]).toBeDefined();
  });

  it("uses the macOS neutral gray palette", () => {
    const c = themes["github-dark"].colors;
    expect(c.bg).toBe("#1d1d1f");
    expect(c.bgPanel).toBe("#2a2a2e");
    expect(c.text).toBe("#f5f5f7");
    expect(c.primary).toBe("#0a84ff");
    expect(c.primaryHover).toBe("#0071e3");
  });

  it("ships a mac-ified light theme", () => {
    const c = themes["github-light"].colors;
    expect(c.bg).toBe("#f5f5f7");
    expect(c.bgPanel).toBe("#ffffff");
    expect(c.text).toBe("#1d1d1f");
  });

  it("defines full 16-color terminal palettes", () => {
    for (const id of ["github-dark", "github-light"]) {
      const t = themes[id]!.colors.terminal;
      for (const key of [
        "background", "foreground", "cursor", "selectionBackground",
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        "brightBlack", "brightRed", "brightGreen", "brightYellow",
        "brightBlue", "brightMagenta", "brightCyan", "brightWhite",
      ]) {
        expect(t[key as keyof typeof t], `${id}.${key}`).toBeTypeOf("string");
      }
    }
  });
});
