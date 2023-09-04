# WARN-Scraper_MA

[Weekly Report Page]: https://www.mass.gov/info-details/worker-adjustment-and-retraining-act-warn-weekly-report

Rust-based scraper the Massachusetts WARN Act Notice [Weekly Report Page].
It has 3 components:

1. A library called `web_scrapper`
   1. It is responsible for scraping the [Weekly Report Page]
   2. It reports back all notices issued for the current fiscal year.
   3. This includes
      1. The weekly data immediately present on the site
      2. Linked data for the current fiscal year
2. A binary called `scrapper_cli` which consumes `web_scrapper`.
   1. It exposes a CLI for users to either
      1. Get all notices
      2. Search through the notices for a specific company
3. A rust-bindings generator called `proto_generator`
   1. It acts as a library which exposes protobuf-based rust stucts
   2. Its purpose is to act as the interface between the scraper library and its consumers
   3. In theory....this could lead to an RPC service version of the scraper with clients.
      1. The clients would be consumed by `scraper_cli`
   4. For now, it just acts to clearly define the schema between producer and consumer.

## Running the Program
[jq]: https://jqlang.github.io/jq/manual/

You must first perform the setup in [Setup Requirements](#setup-requirements).

Then just execute `cargo run` to see your options!

Note that sometimes the notices will be output as json's.
To properly view them, you should pipe the results using [jq].
For example:

```bash
cargo run search <company name> | jq .notices
```

## Setup Requirements

To run the program there are 2 requirements:

[Rust Install Instructions]: https://www.rust-lang.org/tools/install

1. Install cargo via rustup. See [Rust Install Instructions] if needed
2. Install curl
   1. On Debian systems: `sudo apt install curl`
   2. For others, follow the instructions [here](https://help.ubidots.com/en/articles/2165289-learn-how-to-install-run-curl-on-windows-macosx-linux)
