//! Interactive terminal user interface (TUI) for secret management.
//!
//! This module provides a ratatui-based TUI for:
//! - Entering secret key-value pairs interactively
use chrono::{DateTime, Utc};
use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io::{self, Write};

/// A key-value pair representing a GitHub secret.
#[derive(Clone)]
pub struct SecretPair {
    /// The secret key/name.
    pub key: String,
    /// The secret value.
    pub value: String,
}
pub fn prompt_secrets() -> anyhow::Result<Vec<SecretPair>> {
    // Real event source that delegates to `crossterm::event::read`
    struct CrosstermEventSource;
    impl EventSource for CrosstermEventSource {
        fn read_event(&mut self) -> anyhow::Result<Event> {
            Ok(event::read()?)
        }
    }

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Clear the terminal before showing TUI
    execute!(stdout, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut event_src = CrosstermEventSource;
    let res = prompt_secrets_with(&mut terminal, &mut event_src);

    // Restore terminal in all cases
    terminal::disable_raw_mode()?;
    drop(terminal);

    res
}

/// Trait representing an event source (so tests can inject fake events).
pub trait EventSource {
    fn read_event(&mut self) -> anyhow::Result<Event>;
}

/// Prompt the user to enter secret key-value pairs using an injected event source.
pub fn prompt_secrets_with<B: Backend, E: EventSource>(
    terminal: &mut Terminal<B>,
    events: &mut E,
) -> anyhow::Result<Vec<SecretPair>> {
    let mut secrets = Vec::new();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut input_mode = InputMode::Key; // Start with key input
    let mut message = String::new();
    let mut message_color = Color::Yellow;

    loop {
        terminal.draw(|frame| {
            render_secret_input_ui(
                frame,
                &secrets,
                &current_key,
                &current_value,
                &input_mode,
                &message,
                message_color,
            );
        })?;

        // Handle input
        if let Event::Key(key) = events.read_event()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            // Check for Ctrl+C - exit immediately
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                terminal::disable_raw_mode()?;
                std::process::exit(0);
            }

            match input_mode {
                InputMode::Key => {
                    match key.code {
                        KeyCode::Enter => {
                            match crate::validation::validate_secret_key(&current_key) {
                                Ok(()) => {
                                    input_mode = InputMode::Value;
                                    message.clear();
                                }
                                Err(e) => {
                                    message = format!("⚠️  {}", e);
                                    message_color = Color::Yellow;
                                }
                            }
                        }
                        KeyCode::Esc => {
                            if !current_key.is_empty() || !secrets.is_empty() {
                                // Ask for confirmation
                                if confirm_exit_ratatui_with(terminal, events)? {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        KeyCode::Char(c) => {
                            current_key.push(c);
                            message.clear();
                        }
                        KeyCode::Backspace => {
                            current_key.pop();
                            message.clear();
                        }
                        _ => {}
                    }
                }
                InputMode::Value => {
                    match key.code {
                        KeyCode::Enter => {
                            if current_value.trim().is_empty() {
                                message = "⚠️  Value cannot be empty".to_string();
                                message_color = Color::Yellow;
                            } else {
                                // Check for duplicate key
                                let key_to_add = current_key.clone();
                                let was_duplicate = secrets.iter().any(|s| s.key == key_to_add);

                                // Remove existing entry with same key (if any)
                                secrets.retain(|s| s.key != key_to_add);

                                // Add new secret pair
                                secrets.push(SecretPair {
                                    key: current_key.clone(),
                                    value: current_value.clone(),
                                });

                                // Set appropriate message
                                if was_duplicate {
                                    message = format!("✓ Secret '{}' updated", key_to_add);
                                } else {
                                    message = format!("✓ Secret '{}' added", key_to_add);
                                }
                                message_color = Color::Green;
                                current_key.clear();
                                current_value.clear();
                                input_mode = InputMode::Key;
                            }
                        }
                        KeyCode::Esc => {
                            // Go back to key input
                            current_value.clear();
                            input_mode = InputMode::Key;
                            message.clear();
                        }
                        KeyCode::Char(c) => {
                            current_value.push(c);
                            message.clear();
                        }
                        KeyCode::Backspace => {
                            current_value.pop();
                            message.clear();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(secrets)
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Key,
    Value,
}

/// Render the secret input UI using ratatui.
pub fn render_secret_input_ui(
    f: &mut Frame,
    secrets: &[SecretPair],
    current_key: &str,
    current_value: &str,
    input_mode: &InputMode,
    message: &str,
    message_color: Color,
) {
    let size = f.size();

    // Calculate minimum required height for input field (must always be visible)
    // Single input field (3 lines) + message (1 line) = 4 lines minimum
    let min_input_height = 4;
    let header_height = if size.height < 8 { 1 } else { 3 };
    let instructions_height = if size.height < 8 { 1 } else { 3 };
    let fixed_height = header_height + min_input_height + instructions_height;

    // Calculate available space for secrets list (only if terminal is tall enough)
    let available_for_list = if size.height > fixed_height {
        size.height.saturating_sub(fixed_height)
    } else {
        0 // No space for list if terminal is too small
    };

    // Create layout - prioritize input area visibility
    // Use Length constraints for fixed elements to guarantee they're always visible
    let chunks = if available_for_list > 0 {
        // Terminal has enough space - show everything
        Layout::default()
            .constraints([
                Constraint::Length(header_height),       // Header
                Constraint::Max(available_for_list),     // Secrets list (remaining space)
                Constraint::Length(min_input_height), // Input area (always 4 lines: 3 for input + 1 for message)
                Constraint::Length(instructions_height), // Instructions
            ])
            .split(size)
    } else {
        // Terminal too small - show only essential elements (input field)
        Layout::default()
            .constraints([
                Constraint::Length(header_height),       // Minimal header
                Constraint::Length(min_input_height),    // Input area (always 4 lines - guaranteed)
                Constraint::Length(instructions_height), // Minimal instructions
            ])
            .split(size)
    };

    let is_small_terminal = size.height < 11;

    // Header
    if is_small_terminal {
        let header = Paragraph::new("GitHub Secrets (ESC: finish)")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);
    } else {
        let secret_count = secrets.len();
        let header_text = if secret_count == 0 {
            "Enter secret key-value pairs. Press ESC to finish.".to_string()
        } else {
            format!(
                "Enter secret key-value pairs ({} added). Press ESC to finish.",
                secret_count
            )
        };
        let header = Paragraph::new(header_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("GitHub Secrets"),
            )
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);
    }

    // Secrets list (only if we have space and terminal is tall enough)
    let list_chunk_idx = if available_for_list > 0 {
        1
    } else {
        usize::MAX
    }; // Use MAX to indicate no list
    if available_for_list > 0 {
        let mut items = Vec::new();
        for (idx, secret) in secrets.iter().enumerate() {
            let item_text = format!(
                "{}. {} = {}",
                idx + 1,
                secret.key,
                "•".repeat(secret.value.len())
            );
            items.push(ListItem::new(Span::styled(
                item_text,
                Style::default().fg(Color::Green),
            )));
        }
        if items.is_empty() {
            items.push(ListItem::new(Span::styled(
                "No secrets added yet",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let list_title = if secrets.is_empty() {
            "Added Secrets".to_string()
        } else {
            format!("Added Secrets ({})", secrets.len())
        };
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(list_title))
            .style(Style::default().fg(Color::White));
        f.render_widget(list, chunks[list_chunk_idx]);
    }

    // Input area - single field that switches between key and value
    // Input area chunk index depends on whether we're showing the secrets list
    let input_area_chunk_idx = if available_for_list > 0 { 2 } else { 1 };
    let input_chunks = Layout::default()
        .constraints([
            Constraint::Length(3), // Input field (always 3 lines - guaranteed visible)
            Constraint::Length(1), // Message (always 1 line)
        ])
        .split(chunks[input_area_chunk_idx]);

    // Show either key or value input based on mode
    match input_mode {
        InputMode::Key => {
            // Key input with border and cursor
            let key_label = Span::styled(
                "Secret key: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            let key_cursor = "│";
            let key_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Enter Secret Key");
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    key_label,
                    Span::raw(current_key),
                    Span::raw(key_cursor),
                ]))
                .block(key_block),
                input_chunks[0],
            );
        }
        InputMode::Value => {
            // Value input with border and cursor
            let value_label = Span::styled(
                "Secret value: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            let masked_value = "•".repeat(current_value.len());
            let value_cursor = "│";
            let value_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Enter Secret Value");
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    value_label,
                    Span::raw(masked_value),
                    Span::raw(value_cursor),
                ]))
                .block(value_block),
                input_chunks[0],
            );
        }
    }

    // Message
    f.render_widget(
        Paragraph::new(if message.is_empty() { " " } else { message })
            .style(Style::default().fg(message_color))
            .alignment(Alignment::Center),
        input_chunks[1],
    );

    // Instructions with mode-specific hints
    let instructions_chunk_idx = if available_for_list > 0 { 3 } else { 2 };
    let instruction_text = if is_small_terminal {
        match input_mode {
            InputMode::Key => "Enter: next → value | ESC: finish",
            InputMode::Value => "Enter: add secret | ESC: back to key",
        }
    } else {
        match input_mode {
            InputMode::Key => "Enter: confirm key → value input | ESC: finish/cancel",
            InputMode::Value => "Enter: add secret | ESC: back to key | Backspace: delete",
        }
    };
    let instructions = if is_small_terminal {
        Paragraph::new(instruction_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
    } else {
        Paragraph::new(instruction_text)
            .block(Block::default().borders(Borders::ALL).title("Instructions"))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
    };
    f.render_widget(instructions, chunks[instructions_chunk_idx]);
}

/// Confirm exit using ratatui.
// `confirm_exit_ratatui` removed; tests and interactive flows use the injected
// `confirm_exit_ratatui_with` variant which accepts an `EventSource`.
/// Confirm exit using ratatui with an injected event source (for tests).
pub fn confirm_exit_ratatui_with<B: Backend, E: EventSource>(
    terminal: &mut Terminal<B>,
    events: &mut E,
) -> anyhow::Result<bool> {
    let mut cursor_pos = 0; // 0 = Yes, 1 = No

    loop {
        terminal.draw(|frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Length(1)])
                .split(size);

            let options = ["Yes", "No"];
            let mut items = Vec::new();
            for (i, opt) in options.iter().enumerate() {
                let prefix = if cursor_pos == i { "> " } else { "  " };
                let style = if cursor_pos == i {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Span::styled(
                    format!("{}{}", prefix, opt),
                    style,
                )));
            }

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Finish entering secrets? (y/N)"),
                )
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(list, chunks[0]);
        })?;

        if let Event::Key(key) = events.read_event()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            // Check for Ctrl+C - exit immediately
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                terminal::disable_raw_mode()?;
                std::process::exit(0);
            }

            match key.code {
                KeyCode::Left | KeyCode::Up => {
                    cursor_pos = cursor_pos.saturating_sub(1);
                }
                KeyCode::Right | KeyCode::Down => {
                    if cursor_pos < 1 {
                        cursor_pos += 1;
                    }
                }
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter if cursor_pos == 0 => {
                    return Ok(true);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => {
                    return Ok(false);
                }
                KeyCode::Esc => {
                    return Ok(false);
                }
                _ => {}
            }
        }
    }
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

/// Format ISO 8601 date string to human-readable format.
pub fn format_date(date_str: &str) -> String {
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

/// Confirm whether to update an existing secret.
///
/// Displays a confirmation prompt showing the secret name and when it was last updated.
///
/// # Arguments
///
/// * `secret_name` - The name of the secret to update
/// * `last_updated` - Optional ISO 8601 timestamp of when the secret was last updated
///
/// # Returns
///
/// Returns `true` if the user confirms the update, `false` otherwise.
///
/// # Errors
///
/// Returns an error if terminal operations fail.
pub fn confirm_secret_update(
    secret_name: &str,
    last_updated: Option<&str>,
) -> anyhow::Result<bool> {
    print!("\n{}", "⚠️  Secret '".yellow());
    print!("{}", secret_name.bright_yellow());
    print!("{}", "' already exists".yellow());
    if let Some(date) = last_updated {
        let friendly_date = format_date(date);
        print!(
            " {} {}",
            "(last updated:".yellow(),
            friendly_date.bright_yellow()
        );
        print!("{}", ")".yellow());
    }
    print!("{}", ". Overwrite? (y/N): ".yellow());
    io::stdout().flush()?;

    let response = read_single_char()?;
    println!(); // New line after input

    Ok(response == 'y' || response == 'Y')
}

pub fn confirm_retry() -> anyhow::Result<bool> {
    print!(
        "\n{}",
        "Would you like to retry the failed operations? (y/N): ".yellow()
    );
    io::stdout().flush()?;

    let response = read_single_char()?;
    println!(); // New line after input

    Ok(response == 'y' || response == 'Y')
}

/// Present interactive menu for selecting one or more repositories.
/// Returns vector of selected repository indices.
/// Select repositories from a list using an interactive TUI.
///
/// Displays a TUI where users can select multiple repositories using spacebar
/// to toggle selection and Enter to confirm. Includes a "Select All" option.
///
/// # Arguments
///
/// * `repositories` - A slice of repositories to choose from
///
/// # Returns
///
/// Returns a vector of indices representing the selected repositories.
///
/// # Errors
///
/// Returns an error if:
/// - Terminal setup fails
/// - The user cancels the selection (ESC)
/// - Terminal operations fail
pub fn select_repositories(
    repositories: &[crate::config::Repository],
) -> anyhow::Result<Vec<usize>> {
    if repositories.len() == 1 {
        println!(
            "{} {}\n",
            "Using repository:".cyan(),
            repositories[0].display_name().bright_cyan()
        );
        return Ok(vec![0]);
    }

    // Real event source  that delegates to `crossterm::event::read`
    struct CrosstermEventSource;
    impl EventSource for CrosstermEventSource {
        fn read_event(&mut self) -> anyhow::Result<Event> {
            Ok(event::read()?)
        }
    }

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Clear the terminal before showing TUI
    execute!(stdout, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut event_src = CrosstermEventSource;
    let res = select_repositories_with(&mut terminal, &mut event_src, repositories);

    // Restore terminal in all cases
    terminal::disable_raw_mode()?;
    drop(terminal);

    res
}

/// Select repositories with dependency injection for testing.
pub fn select_repositories_with<B: Backend, E: EventSource>(
    terminal: &mut Terminal<B>,
    events: &mut E,
    repositories: &[crate::config::Repository],
) -> anyhow::Result<Vec<usize>> {
    // State: index 0 is "Select All", indices 1.. are repositories
    let mut selected = vec![false; repositories.len() + 1];
    let mut list_state = ListState::default();
    list_state.select(Some(0)); // Start with "Select All" selected

    loop {
        terminal.draw(|frame| {
            render_selection_ui(frame, repositories, &selected, &mut list_state);
        })?;

        // Handle input
        if let Event::Key(key) = events.read_event()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            // Check for Ctrl+C - exit immediately
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                terminal::disable_raw_mode()?;
                std::process::exit(0);
            }

            match key.code {
                KeyCode::Up => {
                    if let Some(selected_idx) = list_state.selected()
                        && selected_idx > 0
                    {
                        list_state.select(Some(selected_idx - 1));
                    }
                }
                KeyCode::Down => {
                    if let Some(selected_idx) = list_state.selected() {
                        if selected_idx < repositories.len() {
                            list_state.select(Some(selected_idx + 1));
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(cursor_pos) = list_state.selected() {
                        if cursor_pos == 0 {
                            // Toggle "Select All"
                            let all_selected = selected[1..].iter().all(|&s| s);
                            let new_state = !all_selected;
                            // Set all repository selections to match "Select All"
                            for selected_item in selected.iter_mut().skip(1) {
                                *selected_item = new_state;
                            }
                            selected[0] = new_state;
                        } else {
                            // Toggle individual repository
                            selected[cursor_pos] = !selected[cursor_pos];
                            // Update "Select All" state based on all repositories
                            let all_selected = selected[1..].iter().all(|&s| s);
                            selected[0] = all_selected;
                        }
                    }
                }
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Esc => {
                    anyhow::bail!("Selection cancelled");
                }
                _ => {}
            }
        }
    }

    // Collect selected repository indices (excluding "Select All" at index 0)
    let selected_indices: Vec<usize> = (1..selected.len())
        .filter(|&i| selected[i])
        .map(|i| i - 1)
        .collect();

    if selected_indices.is_empty() {
        anyhow::bail!("No repositories selected");
    }

    Ok(selected_indices)
}

/// Render the selection UI using ratatui.
pub fn render_selection_ui(
    f: &mut Frame,
    repositories: &[crate::config::Repository],
    selected: &[bool],
    list_state: &mut ListState,
) {
    let size = f.size();

    // Create layout - ensure instructions are always visible
    let min_list_height = 3;
    let instructions_height = 3;
    let available_for_list = size.height.saturating_sub(instructions_height);

    let chunks = Layout::default()
        .constraints([
            Constraint::Max(available_for_list.max(min_list_height)), // List (max available, min 3)
            Constraint::Length(instructions_height), // Instructions (always visible)
        ])
        .split(size);

    // Build list items
    let mut items = Vec::new();

    // "Select All" item
    let checkbox = if selected[0] { "[x]" } else { "[ ]" };
    let select_all_text = format!("{} Select All", checkbox);
    items.push(ListItem::new(select_all_text));

    // Repository items
    for (i, repo) in repositories.iter().enumerate() {
        let idx = i + 1;
        let checkbox = if selected[idx] { "[x]" } else { "[ ]" };
        let repo_text = format!("{} {}", checkbox, repo.display_name());
        items.push(ListItem::new(repo_text));
    }

    // Count selected repositories
    let selected_count: usize = selected[1..].iter().map(|&s| s as usize).sum();
    let total_count = repositories.len();
    let list_title = if selected_count == total_count && selected[0] {
        format!("Repositories (All {} selected)", total_count)
    } else if selected_count > 0 {
        format!(
            "Repositories ({} of {} selected)",
            selected_count, total_count
        )
    } else {
        "Repositories".to_string()
    };

    // Create and render list
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut *list_state);

    // Instructions with selection status
    let instruction_text = if selected_count > 0 {
        format!(
            "↑/↓: navigate | Space: toggle | Enter: confirm ({} selected)",
            selected_count
        )
    } else {
        "↑/↓: navigate | Space: toggle | Enter: confirm".to_string()
    };
    let instructions = Paragraph::new(instruction_text)
        .block(Block::default().borders(Borders::ALL).title("Instructions"))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[1]);
}
