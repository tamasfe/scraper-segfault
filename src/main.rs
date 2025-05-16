use std::sync::Arc;

use eyre::{Context, eyre};
use scraper::{ElementRef, Html, Selector, element_ref::Select};
use tikv_jemallocator::Jemalloc;
use uuid::Uuid;

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

fn main() {
    let test_data = include_str!("../repro.html");

    loop {
        let redacteds = extract_redacted_from_html(test_data);
        match redacteds {
            Ok(redacteds) => {
                for redacted in redacteds {
                    println!("Redacted: {} (ID: {})", redacted.name, redacted.id);
                }
            }
            Err(e) => {
                eprintln!("Error extracting redacteds: {}", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn extract_redacted_from_html(html: &str) -> eyre::Result<Vec<Redacted>> {
    let fragment = Html::parse_fragment(html);
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut redacteds = Vec::new();

    for element in fragment.select(&row_selector) {
        let mut cells = element.select(&cell_selector);
        let Some(redacted_name_cell) = cells.next() else {
            // It's a header row.
            continue;
        };

        match extract_redacted_from_row(cells, redacted_name_cell) {
            Ok(redacted) => {
                redacteds.push(redacted);
            }
            Err(_error) => {}
        }
    }
    Ok(redacteds)
}

fn extract_redacted_from_row(
    mut cells: Select,
    redacted_name_cell: ElementRef,
) -> eyre::Result<Redacted> {
    let redacted_name = redacted_name_cell
        .text()
        .next()
        .ok_or_else(|| eyre!("expected redacted name in cell"))?
        .trim();

    let redacted_format = cells
        .next()
        .ok_or_else(|| eyre!("expected redacted format cell"))?
        .text()
        .next()
        .ok_or_else(|| eyre!("expected redacted format in cell"))?
        .trim();

    let actions_cell = cells
        .next()
        .ok_or_else(|| eyre!("expected cell with run button"))?;

    let button_selector =
        Selector::parse(r#"button[ga-label="run-redacted"], button[name="run-redacted"]"#).unwrap();

    let run_button = actions_cell
        .select(&button_selector)
        .next()
        .ok_or_else(|| eyre!("run redacted button was not found"))?;

    let redacted_id: Uuid = run_button
        .value()
        .attr("value")
        .ok_or_else(|| eyre!("expected redacted ID on button as `value` attribute"))?
        .parse()
        .wrap_err("expected redacted ID to be a valid UUID")?;

    Ok(Redacted {
        name: Arc::from(redacted_name),
        format: Arc::from(redacted_format),
        id: redacted_id,
    })
}

struct Redacted {
    name: Arc<str>,
    format: Arc<str>,
    id: Uuid,
}
