use clap::{Args, Parser, Subcommand};

use proto_generator::{notice_collector::NoticeCollector, notices::WARNNotices};
use web_scraper::scraper::ScraperAdapter;

use protobuf_json_mapping;

const PAGE_BASE_URL: &str = "https://www.mass.gov";
const MAIN_PAGE_URL: &str = "/info-details/worker-adjustment-and-retraining-act-warn-weekly-report";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct ScraperCli {
    #[clap(subcommand)]
    pub(crate) command: ScraperCommands,

    /// Set this flag to make the program verbose
    #[arg(global = true, short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Subcommand, Clone, Debug)]
#[command(author, version, about, short_flag = 'a', long_about = None)]
pub(crate) enum ScraperCommands {
    /// If used, will just display all notices
    GetAll(GetAllNotices),
    Search(SearchForNotice),
}

/// Command to just print all notices
#[derive(Args, Clone, Debug)]
pub(crate) struct GetAllNotices {}

/// Searches through all notices for the provided company name.
/// Print result as json
#[derive(Args, Clone, Debug)]
pub(crate) struct SearchForNotice {
    company_name: String,
}

impl PerformScraping for GetAllNotices {
    fn run_command(&self, is_verbose: bool) {
        let found_notices: WARNNotices = self.do_scraping(is_verbose);
        let merged_notices = found_notices
            .notices
            .into_iter()
            .map(NoticeCollector::to_notices)
            .reduce(NoticeCollector::reduce_notices)
            .unwrap();
        let json_string = protobuf_json_mapping::print_to_string(&merged_notices).unwrap();
        println!("{}", json_string);
    }
}

impl PerformScraping for SearchForNotice {
    fn run_command(&self, is_verbose: bool) {
        let found_notices: WARNNotices = self.do_scraping(is_verbose);
        let found_matches: WARNNotices =
            NoticeCollector::search_notices_for_company(found_notices, &self.company_name);
        let found_json_string = protobuf_json_mapping::print_to_string(&found_matches).unwrap();
        println!("{}", found_json_string);
    }
}

impl ScraperCli {
    pub fn run() {
        let scraper_cli = ScraperCli::parse();
        match scraper_cli.command {
            ScraperCommands::GetAll(notices) => notices.run_command(scraper_cli.verbose),
            ScraperCommands::Search(search) => search.run_command(scraper_cli.verbose),
        }
    }
}

/// Common interface that most command will need to implement to be valid.
/// Also provides common implementation of performing scraping
trait PerformScraping {
    fn do_scraping(&self, is_verbose: bool) -> WARNNotices {
        let scraper_adapter =
            ScraperAdapter::new(PAGE_BASE_URL.to_string(), MAIN_PAGE_URL, is_verbose).unwrap();
        scraper_adapter.get_notices().unwrap()
    }

    /// Each implementers specific way to run a command
    fn run_command(&self, is_verbose: bool);
}
