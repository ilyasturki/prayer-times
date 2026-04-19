mod arguments;
mod calculations;
mod config;
mod daemon;
mod event;
mod location;
mod madhab;
mod method;
mod notification;
mod notification_urgency;
mod prayer;
mod prayers;

use self::{
    arguments::generation::generate, arguments::Arguments, arguments::Commands, config::Config,
    madhab::Madhab, notification::notify_prayer,
};

use clap::Parser;
use daemon::run_daemon;
use method::MethodVariant;

fn main() {
    let args = Arguments::parse();

    let default = Commands::default();
    let command = args.command.as_ref().unwrap_or(&default);
    match command {
        Commands::Daemon(_daemon) => {
            let config = Config::new(&args);
            run_daemon(&config);
        }
        Commands::Previous => {
            let config = Config::new(&args);
            let prayer = prayers::current(&config).previous();
            println!("{}", prayer.text_time());
        }
        Commands::Current => {
            let config = Config::new(&args);
            let prayer = prayers::current(&config);
            println!("{}", prayer.text_time());
        }
        Commands::Next => {
            let config = Config::new(&args);
            let prayer = prayers::next(&config);
            println!("{}", prayer.text_duration());
        }
        Commands::Prayers(list_prayers_args) => {
            let config = Config::new(&args);

            let prayer_list = if let Some(date_str) = &list_prayers_args.date {
                match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    Ok(date) => prayers::list_prayers_for_date(&config, date),
                    Err(_) => {
                        eprintln!("Error: Invalid date format. Please use YYYY-MM-DD format.");
                        std::process::exit(1);
                    }
                }
            } else {
                prayers::list_prayers(&config)
            };

            for prayer in prayer_list {
                println!("{}", prayer.text_time());
            }
        }
        Commands::Methods => {
            MethodVariant::list();
        }
        Commands::Madhab => {
            Madhab::list_all();
        }
        Commands::DryRun => {
            let config = Config::new(&args);
            let next_prayer = prayers::next(&config);
            notify_prayer(&next_prayer, &config);
        }
        Commands::Config => {
            let (program, config) = config::config_options();
            let result = confy::get_configuration_file_path(program, config);
            match result {
                Ok(path) => {
                    println!("Config file: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Error reading config file: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::GenerateShell => generate(),
    }
}
