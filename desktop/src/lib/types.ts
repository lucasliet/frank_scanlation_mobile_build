// Mirrors scanlation_core::db::Manga (serde camelCase).
export interface Manga {
  id: number;
  url: string;
  title: string;
  coverPath: string | null;
  createdAt: number;
  lastReadUrl: string | null;
  lastReadChapter: number | null;
  lastReadAt: number | null;
  latestChapter: number | null;
  latestChapterUrl: string | null;
  lastCheckedAt: number | null;
  hasNew: boolean;
}

export type OpenTarget = "continue" | "home" | "latest";
