<script lang="ts">
  import { onMount } from "svelte";
  import type { Manga } from "./types";
  import { getCover, openManga, removeManga } from "./ipc";
  import { readingStatus } from "./format";

  let {
    manga,
    onchanged
  }: { manga: Manga; onchanged: () => void } = $props();

  let cover: string | null = $state(null);
  let opening = $state(false);

  onMount(async () => {
    try {
      cover = await getCover(manga.id);
    } catch {
      cover = null;
    }
  });

  async function open(target: "continue" | "home" | "latest") {
    opening = true;
    try {
      await openManga(manga.id, target);
    } finally {
      opening = false;
    }
  }

  async function remove() {
    if (!confirm(`Remove "${manga.title}" from the library?`)) {
      return;
    }
    await removeManga(manga.id);
    onchanged();
  }
</script>

<article class="card" class:has-new={manga.hasNew}>
  <button
    class="cover"
    onclick={() => open("continue")}
    disabled={opening}
    title="Continue reading"
  >
    {#if cover}
      <img src={cover} alt={manga.title} loading="lazy" />
    {:else}
      <span class="cover-fallback">{manga.title.slice(0, 2)}</span>
    {/if}
    {#if manga.hasNew}
      <span class="badge">NEW</span>
    {/if}
  </button>

  <div class="meta">
    <h2 title={manga.title}>{manga.title}</h2>
    <p class="status">{readingStatus(manga.lastReadChapter, manga.latestChapter)}</p>
    <div class="actions">
      <button class="primary" onclick={() => open("continue")} disabled={opening}>
        {opening ? "Opening…" : "Continue ▶"}
      </button>
      <button onclick={() => open("latest")} title="Open the newest chapter">Latest</button>
      <button onclick={() => open("home")} title="Open the site's homepage">Site</button>
      <button class="danger" onclick={remove} title="Remove from library">✕</button>
    </div>
  </div>
</article>

<style>
  .card {
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    transition: border-color 120ms ease;
  }

  .card.has-new {
    border-color: var(--new);
  }

  .cover {
    position: relative;
    display: block;
    width: 100%;
    aspect-ratio: 5 / 7;
    padding: 0;
    border: none;
    border-radius: 0;
    background: var(--bg-hover);
    overflow: hidden;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: transform 160ms ease;
  }

  .cover:hover img {
    transform: scale(1.03);
  }

  .cover-fallback {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    font-size: 3rem;
    font-weight: 700;
    color: var(--text-dim);
    text-transform: uppercase;
  }

  .badge {
    position: absolute;
    top: 8px;
    right: 8px;
    background: var(--new);
    color: #0c130f;
    font-size: 0.72rem;
    font-weight: 700;
    padding: 3px 8px;
    border-radius: 999px;
    letter-spacing: 0.04em;
  }

  .meta {
    padding: 10px 12px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  h2 {
    margin: 0;
    font-size: 0.95rem;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    min-height: 2.4em;
  }

  .status {
    margin: 0;
    font-size: 0.8rem;
    color: var(--text-dim);
  }

  .actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .actions button {
    padding: 5px 9px;
    font-size: 0.8rem;
  }

  @media (max-width: 640px) {
    .meta {
      padding: 12px;
      gap: 8px;
    }

    h2 {
      font-size: 1rem;
    }

    .actions button.primary {
      flex: 1 0 100%;
    }

    .actions button:not(.primary) {
      flex: 1 1 0;
    }
  }
</style>
