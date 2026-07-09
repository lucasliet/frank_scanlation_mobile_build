import { invoke } from "@tauri-apps/api/core";
import type { Manga, OpenTarget } from "./types";

export function listManga(): Promise<Manga[]> {
  return invoke("list_manga");
}

export function addManga(url: string): Promise<Manga> {
  return invoke("add_manga", { url });
}

export function removeManga(id: number): Promise<void> {
  return invoke("remove_manga", { id });
}

export function getCover(id: number): Promise<string | null> {
  return invoke("get_cover", { id });
}

export function openManga(id: number, target: OpenTarget): Promise<void> {
  return invoke("open_manga", { id, target });
}

export function checkUpdates(): Promise<Manga[]> {
  return invoke("check_updates");
}
