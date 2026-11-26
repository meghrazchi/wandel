import { describe, expect, it } from "vitest";
import { t } from "./i18n";

const EXPECTED_KEYS = [
  "subtitle",
  "watch",
  "history",
  "dropTitle",
  "dropActive",
  "inspecting",
  "addFiles",
  "addFolder",
  "commandPlaceholder",
  "noCommands",
];

describe("i18n", () => {
  it("resolves every EN and DE translation key", () => {
    for (const key of EXPECTED_KEYS) {
      const en = t("en", key);
      const de = t("de", key);
      expect(en).not.toBe(key);
      expect(de).not.toBe(key);
      expect(en.length).toBeGreaterThan(0);
      expect(de.length).toBeGreaterThan(0);
    }
  });

  it("falls back to EN for unknown locale and returns raw key when missing", () => {
    expect(t("fr" as never, "watch")).toBe("Watch");
    expect(t("en", "nonexistent_key_xyz")).toBe("nonexistent_key_xyz");
  });
});
