use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use github_secrets::prompt::{EventSource, prompt_secrets_with};
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
            // Return a dummy event or error if we run out of events
            // For this test, we expect the interaction to finish before running out
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

fn char_event(c: char) -> Event {
    key_event(KeyCode::Char(c))
}

#[test]
fn test_prompt_secrets_interaction_flow() {
    // Simulate user typing:
    // 1. "KEY1" -> Enter
    // 2. "VAL1" -> Enter
    // 3. Esc (to finish)
    // 4. "y" (confirm exit)

    let events = vec![
        // Type "KEY1"
        char_event('K'),
        char_event('E'),
        char_event('Y'),
        char_event('1'),
        key_event(KeyCode::Enter),
        // Type "VAL1"
        char_event('V'),
        char_event('A'),
        char_event('L'),
        char_event('1'),
        key_event(KeyCode::Enter),
        // Finish
        key_event(KeyCode::Esc),
        // Confirm exit (y)
        char_event('y'),
    ];

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = prompt_secrets_with(&mut terminal, &mut event_source);

    assert!(result.is_ok());
    let secrets = result.unwrap();

    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].key, "KEY1");
    assert_eq!(secrets[0].value, "VAL1");
}

#[test]
fn test_prompt_secrets_validation_error() {
    // Simulate user typing:
    // 1. "KEY@" (invalid char) -> Enter (should fail)
    // 2. Backspace (remove @) -> Enter (should succeed)
    // 3. "VAL" -> Enter
    // 4. Esc -> y

    let events = vec![
        // "KEY@"
        char_event('K'),
        char_event('E'),
        char_event('Y'),
        char_event('@'),
        key_event(KeyCode::Enter), // Should show error
        // Fix it: Backspace
        key_event(KeyCode::Backspace),
        key_event(KeyCode::Enter), // Now "KEY" is valid
        // Value "VAL"
        char_event('V'),
        char_event('A'),
        char_event('L'),
        key_event(KeyCode::Enter),
        // Finish
        key_event(KeyCode::Esc),
        char_event('y'),
    ];

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = prompt_secrets_with(&mut terminal, &mut event_source);

    assert!(result.is_ok());
    let secrets = result.unwrap();

    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].key, "KEY");
    assert_eq!(secrets[0].value, "VAL");
}

#[test]
fn test_prompt_secrets_duplicate_key_update() {
    // Simulate:
    // 1. KEY1 -> VAL1 -> Enter
    // 2. KEY1 -> VAL2 -> Enter (should update)
    // 3. Esc -> y

    let mut events = Vec::new();

    // First entry: KEY1 = VAL1
    for c in "KEY1".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));
    for c in "VAL1".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));

    // Second entry: KEY1 = VAL2
    for c in "KEY1".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));
    for c in "VAL2".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));

    // Finish
    events.push(key_event(KeyCode::Esc));
    events.push(char_event('y'));

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = prompt_secrets_with(&mut terminal, &mut event_source);

    assert!(result.is_ok());
    let secrets = result.unwrap();

    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].key, "KEY1");
    assert_eq!(secrets[0].value, "VAL2");
}

#[test]
fn test_prompt_secrets_esc_from_value_to_key() {
    // Simulate:
    // 1. KEY1 -> Enter (move to value)
    // 2. Esc (back to key)
    // 3. Backspace (edit key) -> "2" -> Enter
    // 4. VAL2 -> Enter
    // 5. Esc -> y

    let mut events = Vec::new();

    // KEY1 -> Enter
    for c in "KEY1".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));

    // Esc (back to key)
    events.push(key_event(KeyCode::Esc));

    // Edit key: Backspace -> "2" -> Enter (KEY2)
    events.push(key_event(KeyCode::Backspace));
    events.push(char_event('2'));
    events.push(key_event(KeyCode::Enter));

    // Value: VAL2 -> Enter
    for c in "VAL2".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));

    // Finish
    events.push(key_event(KeyCode::Esc));
    events.push(char_event('y'));

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = prompt_secrets_with(&mut terminal, &mut event_source);

    assert!(result.is_ok());
    let secrets = result.unwrap();

    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].key, "KEY2");
    assert_eq!(secrets[0].value, "VAL2");
}

#[test]
fn test_prompt_secrets_empty_value_validation() {
    // Simulate:
    // 1. KEY -> Enter
    // 2. Enter (empty value, should fail)
    // 3. "VAL" -> Enter (success)
    // 4. Esc -> y

    let mut events = Vec::new();

    // KEY -> Enter
    for c in "KEY".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));

    // Empty value -> Enter
    events.push(key_event(KeyCode::Enter)); // Should show error

    // VAL -> Enter
    for c in "VAL".chars() {
        events.push(char_event(c));
    }
    events.push(key_event(KeyCode::Enter));

    // Finish
    events.push(key_event(KeyCode::Esc));
    events.push(char_event('y'));

    let mut event_source = MockEventSource::new(events);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let result = prompt_secrets_with(&mut terminal, &mut event_source);

    assert!(result.is_ok());
    let secrets = result.unwrap();

    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].key, "KEY");
    assert_eq!(secrets[0].value, "VAL");
}
