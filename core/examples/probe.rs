//! Debugging probe: run the add-manga extraction pipeline against a
//! live site and print what the app would store.
//!
//! ```bash
//! cargo run -p scanlation-core --example probe -- https://some-scanlation-site.example/
//! ```

use scanlation_core::fetch::Fetcher;
use scanlation_core::resolve_site_info;
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: probe <site-url>");
        std::process::exit(2);
    });
    let base = Url::parse(&arg).expect("valid absolute URL");

    let fetcher = Fetcher::new();
    let (canonical, info) = match resolve_site_info(&fetcher, &base).await {
        Ok(resolved) => resolved,
        Err(e) => {
            eprintln!("fetch failed: {e}");
            std::process::exit(1);
        }
    };

    println!("site:    {canonical}");
    println!("title:   {}", info.title);
    println!("cover:   {}", info.cover_url.as_deref().unwrap_or("(none)"));
    println!("chapters: {} found", info.chapters.len());
    if let Some(first) = info.chapters.first() {
        println!("  first:  ch {} -> {}", first.number, first.url);
    }
    if let Some(latest) = info.latest_chapter() {
        println!("  latest: ch {} -> {}", latest.number, latest.url);
    }
}
