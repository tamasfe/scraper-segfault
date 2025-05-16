use scraper::{Html, Selector};
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

fn main() {
    let test_data = include_str!("../repro.html");

    loop {
        extract_redacted_from_html(test_data);
    }
}

fn extract_redacted_from_html(html: &str) {
    let fragment = Html::parse_fragment(html);
    let row_selector = Selector::parse("tr").unwrap();

    fragment.select(&row_selector).next();
}