use saphyr_parser::{Event, Parser, ScanError};

/// Returns the tags of the scalars in `input`.
fn scalar_tags(input: &str) -> Result<Vec<String>, ScanError> {
    let mut tags = vec![];
    for event in Parser::new_from_str(input) {
        if let (Event::Scalar(_, _, _, Some(tag)), _) = event? {
            tags.push(format!("{}{}", tag.handle, tag.suffix));
        }
    }
    Ok(tags)
}

#[test]
fn multiple_tag_directives() {
    let yaml =
        "%TAG !s! tag:yaml.org,2002:\n%TAG !e! tag:example.com,2000:\n---\n- !s!str a\n- !e!b c";
    assert_eq!(
        scalar_tags(yaml).unwrap(),
        ["tag:yaml.org,2002:str", "tag:example.com,2000:b"]
    );
}

#[test]
fn duplicate_tag_directive() {
    let yaml = "%TAG !e! tag:a,2000:\n%TAG !e! tag:b,2000:\n---\n!e!x y";
    let error = scalar_tags(yaml).unwrap_err();
    assert!(error.info().contains("at most once per handle"), "{error}");
}

#[test]
fn document_versions_are_reported_and_reset_for_every_document() {
    let yaml = "%YAML 1.2\n--- a\n...\n%YAML 1.1\n--- b\n...\n--- c\n";
    for keep_tags in [false, true] {
        let mut parser = Parser::new_from_str(yaml).keep_tags(keep_tags);
        let mut versions = Vec::new();
        while let Some(event) = parser.next_event() {
            if matches!(event.unwrap().0, Event::DocumentStart(_)) {
                versions.push(parser.document_version());
            }
        }
        assert_eq!(versions, [Some((1, 2)), Some((1, 1)), None]);
    }
}

#[test]
fn document_version_reports_newer_directives_without_changing_grammar() {
    let mut parser = Parser::new_from_str("%YAML 1.3\n--- value\n");
    while let Some(event) = parser.next_event() {
        if matches!(event.unwrap().0, Event::DocumentStart(_)) {
            assert_eq!(parser.document_version(), Some((1, 3)));
        }
    }
    let mut parser = Parser::new_from_str("key: '%YAML 1.1'\n");
    while let Some(event) = parser.next_event() {
        event.unwrap();
        assert_eq!(parser.document_version(), None);
    }
}

#[test]
fn cloned_and_peeked_parser_preserves_document_version_metadata() {
    let mut parser = Parser::new_from_str("%YAML 1.2\n--- value\n");
    loop {
        let event = parser.peek().unwrap().unwrap();
        if matches!(event.0, Event::DocumentStart(_)) {
            break;
        }
        parser.next_event().unwrap().unwrap();
    }
    assert_eq!(parser.document_version(), Some((1, 2)));
    let clone = parser.clone();
    assert_eq!(clone.document_version(), Some((1, 2)));
    assert_eq!(parser.collect::<Vec<_>>(), clone.collect::<Vec<_>>());
}
