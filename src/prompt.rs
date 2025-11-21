use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use std::io::{self, Write};
use dialoguer::MultiSelect;

#[derive(Clone)]
pub struct SecretPair {
    pub key: String,
    pub value: String,
}

pub fn prompt_secrets() -> anyhow::Result<Vec<SecretPair>> {
    let mut secrets = Vec::new();
    let mut stdout = io::stdout();

    println!("Enter secret key-value pairs. Press ESC to finish.");
    println!("(Note: ESC key detection requires terminal raw mode)\n");

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
            println!("Key cannot be empty. Skipping...\n");
            continue;
        }

        // Prompt for value
        print!("Secret value: ");
        stdout.flush()?;
        
        let value = read_input_with_esc()?;
        
        if value.is_none() {
            // ESC was pressed, ask for confirmation
            if confirm_exit()? {
                break;
            } else {
                println!("Discarding incomplete secret pair.\n");
                continue;
            }
        }

        let value = value.unwrap();
        if value.trim().is_empty() {
            println!("Value cannot be empty. Skipping...\n");
            continue;
        }

        secrets.push(SecretPair { key, value });
        println!("Secret pair added.\n");
    }

    Ok(secrets)
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
    println!("\n\nAre you sure you want to exit? (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let trimmed = input.trim().to_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

pub fn confirm_secret_update(secret_name: &str, last_updated: Option<&str>) -> anyhow::Result<bool> {
    print!("\nSecret '{}' already exists", secret_name);
    if let Some(date) = last_updated {
        print!(" (last updated: {})", date);
    }
    print!(". Overwrite? (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let trimmed = input.trim().to_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

pub fn confirm_retry() -> anyhow::Result<bool> {
    print!("\nWould you like to retry the failed operations? (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let trimmed = input.trim().to_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

/// Present interactive menu for selecting one or more repositories.
/// Returns vector of selected repository indices.
pub fn select_repositories(repositories: &[crate::config::Repository]) -> anyhow::Result<Vec<usize>> {
    if repositories.len() == 1 {
        println!("Using repository: {}\n", repositories[0].display_name());
        return Ok(vec![0]);
    }

    println!("Select repositories to update secrets for (use Space to select, Enter to confirm):\n");
    
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
        // "Select All" was chosen - select all repositories
        selected_indices = (1..items.len()).collect();
        println!("\nSelected all {} repositories:\n", selected_indices.len());
        for &idx in &selected_indices {
            println!("  - {}", repositories[idx - 1].display_name());
        }
    } else {
        // Map selected menu indices to repository indices
        selected_indices = selections.into_iter()
            .filter(|&i| i > 0)
            .map(|i| i - 1)
            .collect();
        
        println!("\nSelected {} repository/repositories:\n", selected_indices.len());
        for &idx in &selected_indices {
            println!("  - {}", repositories[idx].display_name());
        }
    }
    
    println!();
    Ok(selected_indices)
}

