use serde::Deserialize;
use skim::prelude::*;
use std::{collections::HashMap, fs, io, io::Cursor, process::Command};

#[derive(Debug, Deserialize)]
struct Config {
    username: String,
    dbname: String,
    queries: HashMap<String, String>, // Use a HashMap for alias-query pairs
}

/// Function to display fuzzy selection and return the selected alias and query.
fn select_query(queries: &HashMap<String, String>) -> Option<(String, String)> {
    let query_items: Vec<String> = queries
        .iter()
        .map(|(alias, sql)| format!("{}: {}", alias, sql))
        .collect();

    let joined_items = query_items.join("\n"); // Prepare data for fuzzy selection
    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(joined_items)); // Wrap in a Cursor for `'static` lifetime

    if let Some(output) = Skim::run_with(&options, Some(items)) {
        if !output.is_abort {
            if let Some(selected_item) = output.selected_items.get(0) {
                let output_str = selected_item.output(); // Get the selected item's output
                let selected_alias = output_str.split(":").next().unwrap().to_string();
                if let Some(sql) = queries.get(&selected_alias) {
                    return Some((selected_alias, sql.clone())); // Return alias and query
                }
            }
        }
    }

    None
}

fn execute_query(username: &str, dbname: &str, query: &str) {
    // Print the arguments for debugging
    println!(
        "Executing: csql -u {} {} -S -c \"{}\"",
        username, dbname, query
    );

    // Execute the command using `Command`
    let output = Command::new("csql")
        .arg("-u")
        .arg(username)
        .arg(dbname)
        .arg("-S")
        .arg("-c")
        .arg(query) // Pass the query directly as an argument
        .spawn()
        .expect("Failed to execute csql command");

    println!("Output: {:?}", output);
}

fn main() -> io::Result<()> {
    // Load configuration from TOML file
    let config_content = fs::read_to_string("queries.toml").expect("Failed to read queries.toml");
    let config: Config = toml::from_str(&config_content).expect("Failed to parse queries.toml");

    // Display the fuzzy selector
    if let Some((alias, query)) = select_query(&config.queries) {
        println!("Executing Query: {}", alias);
        // Execute the selected query using `csql`
        execute_query(&config.username, &config.dbname, &query);
    } else {
        println!("No query selected.");
    }

    Ok(())
}
