use rzozowski::Regex;

#[test]
fn test_parse_and_matches() {
    let regex = Regex::from_str("(a|b)*c+").unwrap();
    assert!(regex.matches("c"));
    assert!(regex.matches("cc"));
    assert!(regex.matches("ac"));
    assert!(regex.matches("abc"));
    assert!(regex.matches("abbaccc"));
}
