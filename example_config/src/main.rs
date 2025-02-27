use serde_json::to_string_pretty;
use talmudifier::config::Config;

fn main() {
    let config = Config::default();
    let s = to_string_pretty(&config).unwrap();
    write("example_config.json", s).unwrap();
}