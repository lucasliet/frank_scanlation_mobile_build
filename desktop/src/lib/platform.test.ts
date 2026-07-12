import { describe, expect, test, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: vi.fn(),
}));

import { platform as mockedPlatform } from "@tauri-apps/plugin-os";
import { isAndroid, isMobile } from "./platform";

describe("platform helpers", () => {
  beforeEach(() => {
    vi.mocked(mockedPlatform).mockReset();
  });

  test("isAndroid is true only on android", () => {
    for (const value of ["linux", "macos", "windows", "ios"]) {
      vi.mocked(mockedPlatform).mockReturnValue(value as ReturnType<typeof mockedPlatform>);
      expect(isAndroid()).toBe(false);
    }
    vi.mocked(mockedPlatform).mockReturnValue("android");
    expect(isAndroid()).toBe(true);
  });

  test("isMobile mirrors isAndroid (currently android-only)", () => {
    vi.mocked(mockedPlatform).mockReturnValue("android");
    expect(isMobile()).toBe(true);

    vi.mocked(mockedPlatform).mockReturnValue("ios");
    expect(isMobile()).toBe(false);

    vi.mocked(mockedPlatform).mockReturnValue("linux");
    expect(isMobile()).toBe(false);
  });
});
