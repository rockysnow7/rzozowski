use rzozowski::parse;

#[test]
fn test_parse_and_matches() {
    let regex = parse("(a|b)*c+").unwrap();
    assert!(regex.matches("c"));
    assert!(regex.matches("cc"));
    assert!(regex.matches("ac"));
    assert!(regex.matches("abc"));
    assert!(regex.matches("abbaccc"));
}
