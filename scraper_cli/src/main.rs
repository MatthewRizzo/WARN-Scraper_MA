use web_scraper::scraper::ScraperAdapter;
use protobuf_json_mapping;

static MAIN_PAGE_URL: &str =
    "https://www.mass.gov/info-details/worker-adjustment-and-retraining-act-warn-weekly-report";

fn main() {
    let scraper_adapter = ScraperAdapter::new(MAIN_PAGE_URL).unwrap();
    let new_notices = scraper_adapter.get_notices().unwrap();

    for notice in &new_notices.notices {
        println!(
            "Warn Notice: {}",
            protobuf_json_mapping::print_to_string(notice).expect("Error printing to json")
        );
    }
}
