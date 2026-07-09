import { describe, expect, test } from "vitest";
import { formatChapter, readingStatus, timeAgo } from "./format";

describe("formatChapter", () => {
  test("integers and decimals", () => {
    expect(formatChapter(10)).toBe("Ch. 10");
    expect(formatChapter(10.5)).toBe("Ch. 10.5");
    // Letter sub-chapters arrive as x.01/x.02 from the Rust side.
    expect(formatChapter(10.02)).toBe("Ch. 10.02");
  });

  test("null becomes placeholder", () => {
    expect(formatChapter(null)).toBe("—");
  });
});

describe("readingStatus", () => {
  test("nothing known", () => {
    expect(readingStatus(null, null)).toBe("Not started");
  });

  test("never read but latest known", () => {
    expect(readingStatus(null, 12)).toBe("Unread · latest Ch. 12");
  });

  test("caught up", () => {
    expect(readingStatus(12, 12)).toBe("Read up to Ch. 12");
    expect(readingStatus(12, null)).toBe("Read up to Ch. 12");
  });

  test("behind", () => {
    expect(readingStatus(10, 12)).toBe("Ch. 10 of Ch. 12");
  });
});

describe("timeAgo", () => {
  const now = 1_000_000;
  test("buckets", () => {
    expect(timeAgo(null, now)).toBe("never");
    expect(timeAgo(now - 30, now)).toBe("just now");
    expect(timeAgo(now - 600, now)).toBe("10 min ago");
    expect(timeAgo(now - 7200, now)).toBe("2 h ago");
    expect(timeAgo(now - 172800, now)).toBe("2 d ago");
  });
});
