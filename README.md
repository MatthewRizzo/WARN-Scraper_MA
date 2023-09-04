# WARN-Scraper_MA

[Weekly Report Page]: https://www.mass.gov/info-details/worker-adjustment-and-retraining-act-warn-weekly-report

[WARN Act]: https://en.wikipedia.org/wiki/Worker_Adjustment_and_Retraining_Notification_Act_of_1988

## What is a WARN Notice

The WARN Act in the United States requires providers to report layoffs under
certain conditions 60 calendar days before they occur.
The purpose is to give workers time to find a new job.
Failure to report the layoffs can result in fines and other penalities.
See more information on the [WARN Act] Wikipidea page.

Though the concept seems reassuring, what firms often do is discretely submit the
WARN Notice to the appropriate state bodies, but not inform their workers until
the effective date.
Though they do not break the letter of the law, it is definitely not in the WARN
Act's spirit.

Employees would have to constantly check their state's WARN Act reporting
website to keep apprised of their potential layoff 60 days in advance.
This is not convenient.

In comes this project. This project scapes Massachusets' WARN Reporting site
to retrieve all warn notices given in the last week and historical data from the
current fiscal year and collates it.

Users of the project could search for a specific company through its CLI and
see if the company in question has reported layoffs in the next 60 calendar 
days.

## What is this Project

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
