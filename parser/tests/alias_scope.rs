use saphyr_parser::{Event, EventReceiver, Parser};

#[derive(Default)]
struct Sink;
impl<'input> EventReceiver<'input> for Sink {
    fn on_event(&mut self, _: Event<'input>) {}
}

#[test]
fn iterator_rejects_aliases_from_an_earlier_document() {
    let yaml = "---\na: &secret first\n---\nb: *secret\n";
    let events: Result<Vec<_>, _> = Parser::new_from_str(yaml).collect();
    assert!(
        events.is_err(),
        "anchors are document-scoped even through Iterator"
    );
}

#[test]
fn next_event_rejects_cross_document_aliases_even_when_tags_are_retained() {
    let yaml = "---\na: &secret first\n---\nb: *secret\n";
    let mut parser = Parser::new_from_str(yaml).keep_tags(true);
    let mut error = false;
    while let Some(event) = parser.next_event() {
        if event.is_err() {
            error = true;
            break;
        }
    }
    assert!(
        error,
        "tag retention must not retain another document's anchors"
    );
}

#[test]
fn load_and_same_document_rebinding_keep_their_existing_semantics() {
    let yaml = "---\na: &secret first\n---\nb: *secret\n";
    assert!(Parser::new_from_str(yaml).load(&mut Sink, true).is_err());
    let yaml = "a: &same first\nb: &same second\nc: *same\n";
    let events: Vec<_> = Parser::new_from_str(yaml).map(Result::unwrap).collect();
    let definitions: Vec<_> = events
        .iter()
        .filter_map(|(event, _)| match event {
            Event::Scalar(_, _, id, _) if *id != 0 => Some(*id),
            _ => None,
        })
        .collect();
    let aliases: Vec<_> = events
        .iter()
        .filter_map(|(event, _)| match event {
            Event::Alias(id) => Some(*id),
            _ => None,
        })
        .collect();
    assert_eq!(definitions.len(), 2);
    assert_eq!(aliases, vec![definitions[1]]);
}
