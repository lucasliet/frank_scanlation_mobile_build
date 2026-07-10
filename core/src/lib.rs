//! Site-agnostic scanlation reader core.
//!
//! Everything here is pure Rust with no Tauri dependency:
//! - [`heuristics`]: chapter-number detection ported from the
//!   Prettify Manga Reader browser extension (content.js). The JS and
//!   Rust sides must stay in sync — both parse the same URLs, one to
//!   drive the reader overlay, the other to record reading progress.
//! - [`extract`]: pulls title / cover / chapter list out of a site's
//!   HTML without any site-specific selectors.
//! - [`db`]: the embedded SQLite library (manga list + read state).
//! - [`fetch`]: a small HTTP client used to add manga and to poll for
//!   new chapters in the background.

pub mod db;
pub mod extract;
pub mod fetch;
pub mod heuristics;
pub mod mangadex;

pub use db::{Library, Manga};
pub use extract::SiteInfo;
pub use heuristics::ChapterInfo;

/// Fetch a site's [`SiteInfo`] for any supported URL: API-driven sites
/// with a host-gated adapter (MangaDex), or the generic HTML heuristics
/// for everything else. Returns the possibly-canonicalized site URL
/// alongside (pasting a MangaDex chapter link resolves to its title).
pub async fn resolve_site_info(
    fetcher: &fetch::Fetcher,
    url: &url::Url,
) -> Result<(url::Url, SiteInfo)> {
    if mangadex::is_mangadex_host(url) {
        let title_id = match mangadex::title_id_from_url(url) {
            Some(id) => id,
            None => match mangadex::chapter_id_from_url(url) {
                Some(chapter_id) => mangadex::title_id_for_chapter(fetcher, &chapter_id).await?,
                None => {
                    return Err(Error::Other(
                        "paste a MangaDex title URL (https://mangadex.org/title/…)".into(),
                    ))
                }
            },
        };
        let info = mangadex::site_info(fetcher, &title_id, mangadex::DEFAULT_LANGUAGE).await?;
        let canonical = url::Url::parse(&mangadex::title_url(&title_id))?;
        return Ok((canonical, info));
    }

    let html = fetcher.get_text(url.as_str()).await?;
    Ok((url.clone(), extract::extract_site_info(&html, url)))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid url: {0}")]
    Url(#[from] url::ParseError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
