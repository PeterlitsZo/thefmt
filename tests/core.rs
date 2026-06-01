use indoc::indoc;
use thefmt::format_markdown;

#[test]
fn formats_common_block_and_inline_markdown() {
    let input = indoc! {r#"
        # Title

        Hello *em* and **strong** with [link](https://example.com).

        > quoted

        - one
        - two

        ```rust
        fn main() {}
        ```

        ---
    "#}
    .trim_end();

    let formatted = format_markdown(input).unwrap();

    assert_eq!(
        formatted,
        indoc! {r#"
            # Title

            Hello *em* and **strong** with [link](https://example.com).

            > quoted

            - one
            - two

            ```rust
            fn main() {}
            ```

            ---
        "#}
    );
}

#[test]
fn formats_images_inline_code_and_hard_breaks() {
    let input = indoc! {r#"
        ![alt](image.png "title")

        Use `code`.\
        Next line.
    "#}
    .trim_end();

    let formatted = format_markdown(input).unwrap();

    assert_eq!(
        formatted,
        indoc! {r#"
            ![alt](image.png "title")

            Use `code`.\
            Next line.
        "#}
    );
}
