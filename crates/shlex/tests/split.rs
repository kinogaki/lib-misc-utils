use shlex::{Shlex, split};

#[test]
fn posix_word_splitting() {
    let cases: &[(&str, Option<&[&str]>)] = &[
        ("foo bar", Some(&["foo", "bar"])),
        ("  -O2\t-g \n -DX=1 ", Some(&["-O2", "-g", "-DX=1"])),
        ("'a b' \"c d\" e\\ f", Some(&["a b", "c d", "e f"])),
        ("\"\\$x \\\" \\\\ \\n\"", Some(&["$x \" \\ \\n"])),
        ("'\\n'", Some(&["\\n"])),
        ("a'b'\"c\"d", Some(&["abcd"])),
        ("''", Some(&[""])),
        ("x # comment\ny", Some(&["x", "y"])),
        ("x#notcomment", Some(&["x#notcomment"])),
        ("a\\\nb", Some(&["ab"])),
        ("", Some(&[])),
        ("'unterminated", None),
        ("\"unterminated", None),
        ("trailing\\", None),
    ];
    for (input, expected) in cases {
        let got = split(input);
        let expected = expected.map(|e| e.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(got, expected, "{input:?}");
    }
    let mut lexer = Shlex::new("ok 'bad");
    assert_eq!(lexer.next().as_deref(), Some("ok"));
    assert_eq!(lexer.next(), None);
    assert!(lexer.had_error);
}
