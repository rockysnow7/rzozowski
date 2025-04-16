use rzozowski::Regex;

#[test]
fn test_parse_and_matches() {
    let regex = Regex::new("(a|b)*c+").unwrap();
    assert!(regex.matches("c"));
    assert!(regex.matches("cc"));
    assert!(regex.matches("ac"));
    assert!(regex.matches("abc"));
    assert!(regex.matches("abbaccc"));
}

#[test]
fn test_equality() {
    let r = Regex::new(r"\d{3,6}[a-z_]+").unwrap();
    assert!(r.matches("123abc"));

    let der = r.derivative('1');
    assert_eq!(der, Regex::new(r"\d{2,5}[a-z_]+").unwrap());
}
