use ri::{
    app::Mode,
    token::{get_tokens_from_chars, NormalToken, OperatorToken, RangeToken, Token},
};

#[test]
fn get_chars_empty_set() {
    let chars = vec![];
    let tokens = get_tokens_from_chars(&Mode::Normal, &chars);
    assert_eq!(tokens, vec![])
}

#[test]
fn get_chars_invalid_set() {
    let chars = vec!['*', '2', '#'];
    let tokens = get_tokens_from_chars(&Mode::Normal, &chars);
    assert_eq!(tokens, vec![])
}

#[test]
fn get_chars_single_char() {
    let chars = vec!['k'];
    let tokens = get_tokens_from_chars(&Mode::Normal, &chars);
    assert_eq!(tokens, vec![Token::Normal(NormalToken::Up)])
}

#[test]
fn get_chars_delete_till_eol() {
    let chars = vec!['d', '$'];
    let tokens = get_tokens_from_chars(&Mode::Normal, &chars);
    assert_eq!(
        tokens,
        vec![
            Token::Operator(OperatorToken::Delete),
            Token::Normal(NormalToken::Last)
        ]
    )
}
