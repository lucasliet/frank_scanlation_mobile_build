//! Host-gated MangaDex adapter.
//!
//! MangaDex is an API-driven SPA: the HTML the site serves is an empty
//! shell, so the generic extraction heuristics see nothing, and chapter
//! URLs (`/chapter/<uuid>`) carry no chapter numbers. Mirroring the
//! browser extension (which host-gates a MangaDex reader adapter), this
//! module fills [`SiteInfo`] from the public MangaDex API instead —
//! title, cover, and the chapter feed, English by default.
//!
//! The API and the cover CDN both reject browser-imitation User-Agents
//! from non-browser clients and require an identifying one, hence
//! [`APP_USER_AGENT`].

use crate::extract::{ChapterLink, SiteInfo};
use crate::fetch::Fetcher;
use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::OnceLock;
use url::Url;

pub const APP_USER_AGENT: &str = concat!(
    "FrankScanlation/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/akitaonrails/frank_scanlation)"
);
pub const DEFAULT_LANGUAGE: &str = "en";

const API_BASE: &str = "https://api.mangadex.org";
const FEED_PAGE_LIMIT: usize = 500;
/// Hard cap on feed pagination — protects against a runaway loop if the
/// API ever misreports `total`.
const MAX_FEED_PAGES: usize = 20;

fn uuid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .expect("uuid regex")
    })
}

/// True for mangadex.org and its subdomains (uploads., api., www.).
pub fn is_mangadex_host(url: &Url) -> bool {
    matches!(url.host_str(), Some(host) if host == "mangadex.org" || host.ends_with(".mangadex.org"))
}

fn path_uuid(url: &Url, prefix: &str) -> Option<String> {
    let mut segments = url.path_segments()?;
    if segments.next()? != prefix {
        return None;
    }
    let candidate = segments.next()?.to_lowercase();
    uuid_re().is_match(&candidate).then_some(candidate)
}

/// `https://mangadex.org/title/<uuid>/<slug>` → uuid.
pub fn title_id_from_url(url: &Url) -> Option<String> {
    if !is_mangadex_host(url) {
        return None;
    }
    path_uuid(url, "title")
}

/// `https://mangadex.org/chapter/<uuid>[/<page>]` → uuid.
pub fn chapter_id_from_url(url: &Url) -> Option<String> {
    if !is_mangadex_host(url) {
        return None;
    }
    path_uuid(url, "chapter")
}

pub fn chapter_page_url(chapter_id: &str) -> String {
    format!("https://mangadex.org/chapter/{chapter_id}")
}

async fn get_api_json(fetcher: &Fetcher, url: &str) -> crate::Result<Value> {
    let body = fetcher.get_text_as(url, APP_USER_AGENT).await?;
    serde_json::from_str(&body)
        .map_err(|e| crate::Error::Other(format!("mangadex api returned non-JSON: {e}")))
}

/// Title + cover + full (deduplicated) chapter list for a MangaDex
/// title, in the requested translation language.
pub async fn site_info(
    fetcher: &Fetcher,
    title_id: &str,
    language: &str,
) -> crate::Result<SiteInfo> {
    let manga = get_api_json(
        fetcher,
        &format!("{API_BASE}/manga/{title_id}?includes%5B%5D=cover_art"),
    )
    .await?;
    let (title, cover_url) = parse_manga(&manga, title_id)
        .ok_or_else(|| crate::Error::Other("mangadex: unexpected /manga response".into()))?;

    let mut chapters: Vec<ChapterLink> = Vec::new();
    let mut seen_numbers: HashSet<u64> = HashSet::new();
    let mut offset = 0;
    for _page in 0..MAX_FEED_PAGES {
        let feed = get_api_json(
            fetcher,
            &format!(
                "{API_BASE}/manga/{title_id}/feed?translatedLanguage%5B%5D={language}\
                 &order%5Bchapter%5D=asc&limit={FEED_PAGE_LIMIT}&offset={offset}"
            ),
        )
        .await?;
        let (mut page_chapters, total, count) = parse_feed(&feed);
        page_chapters.retain(|c| seen_numbers.insert(c.number.to_bits()));
        chapters.extend(page_chapters);
        offset += count;
        if count == 0 || offset >= total {
            break;
        }
    }
    chapters.sort_by(|a, b| a.number.total_cmp(&b.number));

    Ok(SiteInfo {
        title,
        cover_url,
        chapters,
    })
}

/// Chapter number for a `/chapter/<uuid>` navigation, straight from the
/// API (`null` for oneshots without a number).
pub async fn chapter_number(fetcher: &Fetcher, chapter_id: &str) -> crate::Result<Option<f64>> {
    let json = get_api_json(fetcher, &format!("{API_BASE}/chapter/{chapter_id}")).await?;
    Ok(parse_chapter_number(&json))
}

/// Resolve the owning title uuid for a pasted chapter URL, so adding a
/// manga by chapter link still works.
pub async fn title_id_for_chapter(fetcher: &Fetcher, chapter_id: &str) -> crate::Result<String> {
    let json = get_api_json(fetcher, &format!("{API_BASE}/chapter/{chapter_id}")).await?;
    json["data"]["relationships"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| r["type"] == "manga")
        .and_then(|r| r["id"].as_str())
        .map(str::to_string)
        .ok_or_else(|| crate::Error::Other("mangadex: chapter has no manga relationship".into()))
}

/// Canonical title URL used as the library entry's site URL.
pub fn title_url(title_id: &str) -> String {
    format!("https://mangadex.org/title/{title_id}")
}

// ---------- pure parsing (unit tested) ----------

fn parse_manga(json: &Value, title_id: &str) -> Option<(String, Option<String>)> {
    let data = &json["data"];
    let titles = &data["attributes"]["title"];
    let title = titles["en"]
        .as_str()
        .or_else(|| titles["ja-ro"].as_str())
        .map(str::to_string)
        .or_else(|| {
            titles
                .as_object()
                .and_then(|map| map.values().next())
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })?;

    let cover = data["relationships"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| r["type"] == "cover_art")
        .and_then(|r| r["attributes"]["fileName"].as_str())
        .map(|file| format!("https://uploads.mangadex.org/covers/{title_id}/{file}.512.jpg"));

    Some((title, cover))
}

/// Returns (chapters-with-numbers, total, raw-entry-count-in-page).
/// Entries without a chapter number (oneshots) or hosted off-site
/// (externalUrl) are skipped; duplicates (multiple scanlation groups
/// releasing the same chapter) are handled by the caller.
fn parse_feed(json: &Value) -> (Vec<ChapterLink>, usize, usize) {
    let total = json["total"].as_u64().unwrap_or(0) as usize;
    let entries = json["data"].as_array().cloned().unwrap_or_default();
    let count = entries.len();

    let chapters = entries
        .iter()
        .filter(|entry| entry["attributes"]["externalUrl"].is_null())
        .filter_map(|entry| {
            let number: f64 = entry["attributes"]["chapter"].as_str()?.parse().ok()?;
            let id = entry["id"].as_str()?;
            Some(ChapterLink {
                number,
                url: chapter_page_url(id),
            })
        })
        .collect();

    (chapters, total, count)
}

fn parse_chapter_number(json: &Value) -> Option<f64> {
    json["data"]["attributes"]["chapter"].as_str()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn host_gating() {
        assert!(is_mangadex_host(&url("https://mangadex.org/title/x")));
        assert!(is_mangadex_host(&url(
            "https://uploads.mangadex.org/covers/a/b.jpg"
        )));
        assert!(!is_mangadex_host(&url("https://notmangadex.org/")));
        assert!(!is_mangadex_host(&url(
            "https://mangadex.org.evil.example/"
        )));
    }

    #[test]
    fn extracts_title_and_chapter_uuids() {
        assert_eq!(
            title_id_from_url(&url(
                "https://mangadex.org/title/65263bf9-4f87-4513-b72f-ad6436b3911c/wotaku-ni-koi-wa-muzukashii"
            ))
            .as_deref(),
            Some("65263bf9-4f87-4513-b72f-ad6436b3911c")
        );
        assert_eq!(
            chapter_id_from_url(&url(
                "https://mangadex.org/chapter/2827A899-0DC3-4841-9869-01FF9D3F0AE2/5"
            ))
            .as_deref(),
            Some("2827a899-0dc3-4841-9869-01ff9d3f0ae2")
        );
        assert_eq!(
            title_id_from_url(&url("https://mangadex.org/title/not-a-uuid")),
            None
        );
        assert_eq!(
            title_id_from_url(&url(
                "https://other.example/title/65263bf9-4f87-4513-b72f-ad6436b3911c"
            )),
            None
        );
        assert_eq!(chapter_id_from_url(&url("https://mangadex.org/")), None);
    }

    #[test]
    fn parses_manga_with_title_fallback_and_cover() {
        let json: Value = serde_json::from_str(
            r#"{"data":{"attributes":{"title":{"ja-ro":"Wotaku ni Koi wa Muzukashii"}},
                "relationships":[
                  {"type":"author","id":"x"},
                  {"type":"cover_art","id":"y","attributes":{"fileName":"cover.jpg"}}
                ]}}"#,
        )
        .unwrap();
        let (title, cover) = parse_manga(&json, "the-title-id").unwrap();
        assert_eq!(title, "Wotaku ni Koi wa Muzukashii");
        assert_eq!(
            cover.as_deref(),
            Some("https://uploads.mangadex.org/covers/the-title-id/cover.jpg.512.jpg")
        );
    }

    #[test]
    fn parses_manga_preferring_english() {
        let json: Value = serde_json::from_str(
            r#"{"data":{"attributes":{"title":{"en":"Wotakoi","ja-ro":"Wotaku ni"}},"relationships":[]}}"#,
        )
        .unwrap();
        let (title, cover) = parse_manga(&json, "id").unwrap();
        assert_eq!(title, "Wotakoi");
        assert_eq!(cover, None);
    }

    #[test]
    fn parses_feed_skipping_oneshots_and_external() {
        let json: Value = serde_json::from_str(
            r#"{"total":4,"data":[
              {"id":"aaa","attributes":{"chapter":"1","externalUrl":null}},
              {"id":"bbb","attributes":{"chapter":null,"externalUrl":null}},
              {"id":"ccc","attributes":{"chapter":"2","externalUrl":"https://elsewhere.example/"}},
              {"id":"ddd","attributes":{"chapter":"2.5","externalUrl":null}}
            ]}"#,
        )
        .unwrap();
        let (chapters, total, count) = parse_feed(&json);
        assert_eq!(total, 4);
        assert_eq!(count, 4);
        let pairs: Vec<(f64, &str)> = chapters
            .iter()
            .map(|c| (c.number, c.url.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                (1.0, "https://mangadex.org/chapter/aaa"),
                (2.5, "https://mangadex.org/chapter/ddd")
            ]
        );
    }

    #[test]
    fn parses_chapter_number_and_oneshot() {
        let json: Value =
            serde_json::from_str(r#"{"data":{"attributes":{"chapter":"104.5"}}}"#).unwrap();
        assert_eq!(parse_chapter_number(&json), Some(104.5));
        let oneshot: Value =
            serde_json::from_str(r#"{"data":{"attributes":{"chapter":null}}}"#).unwrap();
        assert_eq!(parse_chapter_number(&oneshot), None);
    }
}
