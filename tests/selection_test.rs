use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use github_secrets::config::Repository;
use github_secrets::prompt::{EventSource, select_repositories_with};
use ratatui::{Terminal, backend::TestBackend};
use std::collections::VecDeque;

struct MockEventSource {
    events: VecDeque<Event>,
}

impl MockEventSource {
    fn new(events: Vec<Event>) -> Self {
        Self {
            events: VecDeque::from(events),
        }
    }
}

impl EventSource for MockEventSource {
    fn read_event(&mut self) -> anyhow::Result<Event> {
        if let Some(event) = self.events.pop_front() {
            Ok(event)
        } else {
            Ok(Event::Key(KeyEvent::new(
                KeyCode::Null,
                KeyModifiers::empty(),
            )))
        }
    }
}

fn key_event(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    })
}

#[test]
fn test_select_repositories_select_all() {
    let repos = vec![
        Repository {
            owner: "owner1".to_string(),
            name: "repo1".to_string(),
            alias: None,
        },
        Repository {
            owner: "owner2".to_string(),
            name: "repo2".to_string(),
            alias: None,
        },
    ];

    let events = vec![key_event(KeyCode::Char(' ')), key_event(KeyCode::Enter)];

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = select_repositories_with(&mut terminal, &mut event_source, &repos);

    assert!(result.is_ok());
    let selected = result.unwrap();
    assert_eq!(selected.len(), 2);
    assert_eq!(selected, vec![0, 1]);
}

#[test]
fn test_select_repositories_select_subset() {
    let repos = vec![
        Repository {
            owner: "owner1".to_string(),
            name: "repo1".to_string(),
            alias: None,
        },
        Repository {
            owner: "owner2".to_string(),
            name: "repo2".to_string(),
            alias: None,
        },
        Repository {
            owner: "owner3".to_string(),
            name: "repo3".to_string(),
            alias: None,
        },
    ];

    let events = vec![
        key_event(KeyCode::Down),      // Move to repo1
        key_event(KeyCode::Char(' ')), // Select repo1
        key_event(KeyCode::Down),      // Move to repo2
        key_event(KeyCode::Char(' ')), // Select repo2
        key_event(KeyCode::Enter),     // Confirm
    ];

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = select_repositories_with(&mut terminal, &mut event_source, &repos);

    assert!(result.is_ok());
    let selected = result.unwrap();
    assert_eq!(selected.len(), 2);
    assert_eq!(selected, vec![0, 1]);
}

#[test]
fn test_select_repositories_cancel() {
    let repos = vec![Repository {
        owner: "owner1".to_string(),
        name: "repo1".to_string(),
        alias: None,
    }];

    let events = vec![
        key_event(KeyCode::Esc), // Esc to cancel
    ];

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = select_repositories_with(&mut terminal, &mut event_source, &repos);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Selection cancelled")
    );
}

#[test]
fn test_select_repositories_no_selection() {
    let repos = vec![Repository {
        owner: "owner1".to_string(),
        name: "repo1".to_string(),
        alias: None,
    }];

    let events = vec![
        key_event(KeyCode::Enter), // Just Enter without selecting anything
    ];

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = select_repositories_with(&mut terminal, &mut event_source, &repos);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No repositories selected")
    );
}
