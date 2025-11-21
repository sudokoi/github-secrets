use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use std::io::{self, Write};
use dialoguer::MultiSelect;
use chrono::{DateTime, Utc};
use colored::*;

#[derive(Clone)]
pub struct SecretPair {
    pub key: String,
    pub value: String,
}

pub fn prompt_secrets() -> anyhow::Result<Vec<SecretPair>> {
    let mut secrets = Vec::new();
    let mut stdout = io::stdout();

    println!("{}", "Enter secret key-value pairs. Press ESC to finish.".cyan());
    println!("{}", "(Note: ESC key detection requires terminal raw mode)\n".bright_black());

    loop {
        // Prompt for key
        print!("Secret key (or ESC to finish): ");
        stdout.flush()?;
        
        let key = read_input_with_esc()?;
        
        if key.is_none() {
            // ESC was pressed, ask for confirmation
            if confirm_exit()? {
                break;
            } else {
                continue;
            }
        }

        let key = key.unwrap();
        if key.trim().is_empty() {
            println!("{}", "⚠️  Key cannot be empty. Skipping...\n".yellow());
            continue;
        }

        print!("{}", "Secret value: ".cyan());
        stdout.flush()?;
        
        let value = read_input_with_esc()?;
        
        if value.is_none() {
            if confirm_exit()? {
                break;
            } else {
                println!("{}", "⚠️  Discarding incomplete secret pair.\n".yellow());
                continue;
            }
        }

        let value = value.unwrap();
        if value.trim().is_empty() {
            println!("{}", "⚠️  Value cannot be empty. Skipping...\n".yellow());
            continue;
        }

        secrets.push(SecretPair { key, value });
        println!("{}", format!("✓ Secret pair added.\n").green());
    }

    Ok(secrets)
}

/// Read a single character from stdin without requiring Enter.
/// Returns the character that was pressed.
fn read_single_char() -> anyhow::Result<char> {
    terminal::enable_raw_mode()?;
    
    let result = loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                ..
            }) => {
                break Ok(c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                // Treat Enter as 'N' (default)
                break Ok('N');
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                // Treat ESC as 'N' (default)
                break Ok('N');
            }
            _ => {}
        }
    };
    
    terminal::disable_raw_mode()?;
    result
}

/// Read user input with ESC key detection for early exit.
/// Returns None if ESC was pressed, Some(input) if Enter was pressed.
fn read_input_with_esc() -> anyhow::Result<Option<String>> {
    terminal::enable_raw_mode()?;
    let mut input = String::new();
    
    let result = loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                break Ok(None);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                break Ok(Some(input));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            }) => {
                print!("{}", c);
                io::stdout().flush()?;
                input.push(c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if !input.is_empty() {
                    input.pop();
                    print!("\x08 \x08");
                    io::stdout().flush()?;
                }
            }
            _ => {}
        }
    };
    
    terminal::disable_raw_mode()?;
    println!();
    
    result
}

fn confirm_exit() -> anyhow::Result<bool> {
    print!("\n{}", "Finish entering secrets? (y/N): ".yellow());
    io::stdout().flush()?;
    
    let response = read_single_char()?;
    println!(); // New line after input
    
    Ok(response == 'y' || response == 'Y')
}

/// Format ISO 8601 date string to human-readable format.
fn format_date(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&Utc));
        
        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "just now".to_string()
        }
    } else {
        date_str.to_string()
    }
}

pub fn confirm_secret_update(secret_name: &str, last_updated: Option<&str>) -> anyhow::Result<bool> {
    print!("\n{}", "⚠️  Secret '".yellow());
    print!("{}", secret_name.bright_yellow());
    print!("{}", "' already exists".yellow());
    if let Some(date) = last_updated {
        let friendly_date = format_date(date);
        print!(" {} {}", "(last updated:".yellow(), friendly_date.bright_yellow());
        print!("{}", ")".yellow());
    }
    print!("{}", ". Overwrite? (y/N): ".yellow());
    io::stdout().flush()?;
    
    let response = read_single_char()?;
    println!(); // New line after input
    
    Ok(response == 'y' || response == 'Y')
}

pub fn confirm_retry() -> anyhow::Result<bool> {
    print!("\n{}", "Would you like to retry the failed operations? (y/N): ".yellow());
    io::stdout().flush()?;
    
    let response = read_single_char()?;
    println!(); // New line after input
    
    Ok(response == 'y' || response == 'Y')
}

/// Present interactive menu for selecting one or more repositories.
/// Returns vector of selected repository indices.
pub fn select_repositories(repositories: &[crate::config::Repository]) -> anyhow::Result<Vec<usize>> {
    if repositories.len() == 1 {
        println!("{} {}\n", "Using repository:".cyan(), repositories[0].display_name().bright_cyan());
        return Ok(vec![0]);
    }

    println!("{}", "Select repositories to update secrets for (use Space to select, Enter to confirm):\n".cyan());
    
    let mut items: Vec<String> = repositories
        .iter()
        .map(|r| r.display_name())
        .collect();
    
    items.insert(0, "Select All".to_string());

    let selections = MultiSelect::new()
        .with_prompt("Repositories")
        .items(&items)
        .defaults(&vec![false; items.len()])
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to select repositories: {}", e))?;

    if selections.is_empty() {
        anyhow::bail!("No repositories selected");
    }

    let selected_indices: Vec<usize>;
    
    if selections.contains(&0) {
        selected_indices = (1..items.len()).collect();
        println!("\n{} {} {}:\n", 
            "✓".green(), 
            format!("Selected all {} repositories", selected_indices.len()).green(),
            "✓".green());
        for &idx in &selected_indices {
            println!("  {} {}", "•".green(), repositories[idx - 1].display_name().bright_green());
        }
    } else {
        selected_indices = selections.into_iter()
            .filter(|&i| i > 0)
            .map(|i| i - 1)
            .collect();
        
        println!("\n{} {} {}:\n", 
            "✓".green(),
            format!("Selected {} repository/repositories", selected_indices.len()).green(),
            "✓".green());
        for &idx in &selected_indices {
            println!("  {} {}", "•".green(), repositories[idx].display_name().bright_green());
        }
    }
    
    println!();
    Ok(selected_indices)
}

