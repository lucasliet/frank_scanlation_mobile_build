// Small display helpers for the library grid. Pure functions — unit
// tested in format.test.ts.

/** "Ch. 10" / "Ch. 10.5"; null → em-dash placeholder. */
export function formatChapter(n: number | null): string {
  if (n === null || Number.isNaN(n)) {
    return "—";
  }
  const rounded = Math.round(n * 100) / 100;
  return `Ch. ${rounded}`;
}

/** One-line reading status for a manga card. */
export function readingStatus(
  lastRead: number | null,
  latest: number | null
): string {
  if (lastRead === null && latest === null) {
    return "Not started";
  }
  if (lastRead === null) {
    return `Unread · latest ${formatChapter(latest)}`;
  }
  if (latest === null || latest <= lastRead) {
    return `Read up to ${formatChapter(lastRead)}`;
  }
  return `${formatChapter(lastRead)} of ${formatChapter(latest)}`;
}

/** Coarse relative time for "checked X ago" hints. */
export function timeAgo(unixSecs: number | null, nowSecs = Date.now() / 1000): string {
  if (unixSecs === null || unixSecs <= 0) {
    return "never";
  }
  const delta = Math.max(0, nowSecs - unixSecs);
  if (delta < 90) return "just now";
  if (delta < 3600) return `${Math.round(delta / 60)} min ago`;
  if (delta < 86400) return `${Math.round(delta / 3600)} h ago`;
  return `${Math.round(delta / 86400)} d ago`;
}
