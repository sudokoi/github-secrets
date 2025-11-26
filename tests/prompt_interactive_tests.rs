use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};

use github_secrets::prompt::{self, EventSource};

struct FakeEventSource {
    events: Vec<Event>,
    idx: usize,
}

impl FakeEventSource {
    fn new(events: Vec<Event>) -> Self {
        Self { events, idx: 0 }
    }
}

impl EventSource for FakeEventSource {
    fn read_event(&mut self) -> anyhow::Result<Event> {
        if self.idx >= self.events.len() {
            // If we run out of events, return a harmless ESC press
            Ok(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)))
        } else {
            let ev = self.events[self.idx].clone();
            self.idx += 1;
            Ok(ev)
        }
    }
}

#[test]
fn test_prompt_secrets_with_fake_events() {
    // Simulate: type K E Y, Enter => type v a l, Enter => Esc + 'y' to confirm exit
    let mut events = vec![
        Event::Key(KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
    ];

    // Ensure KeyEventKind::Press for those keys where prompt checks it
    // Build events with KeyEventKind::Press explicitly
    events = events
        .into_iter()
        .map(|ev| match ev {
            Event::Key(mut ke) => {
                ke.kind = KeyEventKind::Press;
                Event::Key(ke)
            }
            other => other,
        })
        .collect();

    let mut src = FakeEventSource::new(events);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let secrets = prompt::prompt_secrets_with(&mut terminal, &mut src).unwrap();

    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].key, "KEY");
    assert_eq!(secrets[0].value, "val");
}
