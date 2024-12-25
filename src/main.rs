use serde::Deserialize;
use skim::prelude::*;
use std::{
    fs,
    io::{self, Cursor},
};

#[derive(Debug, Deserialize)]
struct Config {
    queries: Vec<Query>,
}

#[derive(Debug, Deserialize, Clone)]
struct Query {
    alias: String,
    sql: String,
}

/// Function to display fuzzy selection and return the selected query.
fn select_query(queries: &[Query]) -> Option<Query> {
    let query_items: Vec<String> = queries
        .iter()
        .map(|q| format!("{}: {}", q.alias, q.sql))
        .collect();

    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let joined_items = query_items.join("\n");
    let items = item_reader.of_bufread(Cursor::new(joined_items)); // Wrap in a Cursor

    if let Some(output) = Skim::run_with(&options, Some(items)) {
        if !output.is_abort {
            let get = output.selected_items.first();
            if let Some(selected_item) = get {
                let output_str = selected_item.output(); // Extend lifetime of output
                let selected_alias = output_str.split(":").next().unwrap();
                return queries.iter().find(|q| q.alias == selected_alias).cloned();
            }
        }
    }

    None
}

fn main() -> io::Result<()> {
    // Load configuration from YAML file
    let config_content = fs::read_to_string("config.yml").expect("Failed to read config.yml");
    let config: Config = serde_yaml::from_str(&config_content).expect("Failed to parse config.yml");

    // Display the fuzzy selector
    if let Some(selected_query) = select_query(&config.queries) {
        // Print the selected query
        println!("Selected Query:");
        println!("Alias: {}", selected_query.alias);
        println!("SQL: {}", selected_query.sql);
    } else {
        println!("No query selected.");
    }

    Ok(())
}
