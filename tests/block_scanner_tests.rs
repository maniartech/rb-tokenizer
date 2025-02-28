use rb_tokenizer::{Tokenizer, TokenizerConfig};

fn get_block_scanner_tokenizer() -> Tokenizer {
    let config = TokenizerConfig {
        tokenize_whitespace: true,
        continue_on_error: true,
        error_tolerance_limit: 5,
        track_token_positions: true,
    };
    let mut tokenizer = Tokenizer::with_config(config);

    // Add block scanners for different use cases

    // Block comments with /* ... */
    tokenizer.add_block_scanner(
        "/*",
        "*/",
        "Comment",
        Some("BlockComment"),
        false,
        false,
        true
    );

    // Code blocks with { ... }
    tokenizer.add_block_scanner(
        "{",
        "}",
        "CodeBlock",
        None,
        true, // Allow nesting for code blocks
        false,
        true
    );

    // XML/HTML-style tags with < ... >
    tokenizer.add_block_scanner(
        "<",
        ">",
        "Tag",
        None,
        false,
        false,
        true
    );

    // Raw string literals with r" ... "
    tokenizer.add_block_scanner(
        "r\"",
        "\"",
        "String",
        Some("RawString"),
        false,
        true, // Raw mode enabled
        true
    );

    // Add regular scanners for other tokens
    tokenizer.add_regex_scanner(r"^[a-zA-Z_][a-zA-Z0-9_]*", "Identifier", None);
    tokenizer.add_regex_scanner(r"^\d+", "Number", None);
    tokenizer.add_symbol_scanner(";", "Semicolon", None);

    tokenizer
}

#[cfg(test)]
mod block_scanner_tests {
    use super::*;

    #[test]
    fn test_simple_block_comments() {
        let tokenizer = get_block_scanner_tokenizer();

        let input = "/* This is a block comment */ var";
        let result = tokenizer.tokenize(input).expect("Tokenization failed");

        // Expected tokens: BlockComment, Whitespace, Identifier
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].token_type, "Comment");
        assert_eq!(result[0].token_sub_type, Some("BlockComment".to_string()));
        assert_eq!(result[0].value, "/* This is a block comment */");

        // Check positions
        assert_eq!(result[0].line, 1);
        assert_eq!(result[0].column, 1);
    }

    #[test]
    fn test_nested_code_blocks() {
        let tokenizer = get_block_scanner_tokenizer();

        let input = "{ outer { inner } block }";
        let result = tokenizer.tokenize(input).expect("Tokenization failed");

        // Expected tokens: CodeBlock
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token_type, "CodeBlock");
        assert_eq!(result[0].value, "{ outer { inner } block }");

        // Verify nesting worked correctly
        assert!(result[0].value.contains("{ inner }"));
    }

    #[test]
    fn test_raw_string_literals() {
        let tokenizer = get_block_scanner_tokenizer();

        // Use a simpler raw string format for testing
        let input = r#"r"This is a raw string with \n and \t escape sequences";"#;
        let result = tokenizer.tokenize(input).expect("Tokenization failed");

        // Expected tokens: String, Semicolon
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].token_type, "String");
        assert_eq!(result[0].token_sub_type, Some("RawString".to_string()));

        // Check that escape sequences are preserved intact
        assert!(result[0].value.contains("\\n"));
        assert!(result[0].value.contains("\\t"));
    }

    #[test]
    fn test_html_tags() {
        let tokenizer = get_block_scanner_tokenizer();

        let input = "<div>content</div>";
        let result = tokenizer.tokenize(input).expect("Tokenization failed");

        // Expected tokens: Tag, Identifier, Tag
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].token_type, "Tag");
        assert_eq!(result[0].value, "<div>");
        assert_eq!(result[1].token_type, "Identifier");
        assert_eq!(result[1].value, "content");
        assert_eq!(result[2].token_type, "Tag");
        assert_eq!(result[2].value, "</div>");
    }

    #[test]
    fn test_unmatched_block_delimiter() {
        let tokenizer = get_block_scanner_tokenizer();

        // Missing closing comment delimiter
        let input = "/* This comment is not closed properly var";
        let result = tokenizer.tokenize(input);

        // Should return an error
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(!errors.is_empty());
            // Just check that we got some error, not the exact format
            println!("Error type: {:?}", errors[0]);
        }
    }

    #[test]
    fn test_complex_mixed_content() {
        let tokenizer = get_block_scanner_tokenizer();

        let input = "/* Comment */ { code with /* nested comment */ } <tag>content</tag>";
        let result = tokenizer.tokenize(input).expect("Tokenization failed");

        // Expected tokens: Comment, Whitespace, CodeBlock, Whitespace, Tag, Identifier, Tag
        assert_eq!(result.len(), 7);

        // Verify specific tokens
        assert_eq!(result[0].token_type, "Comment");
        assert_eq!(result[2].token_type, "CodeBlock");
        assert!(result[2].value.contains("/* nested comment */"));
        assert_eq!(result[4].token_type, "Tag");
        assert_eq!(result[5].token_type, "Identifier");
        assert_eq!(result[6].token_type, "Tag");
    }

    #[test]
    fn test_whitespace_in_blocks() {
        let tokenizer = get_block_scanner_tokenizer();

        let input = "{\n  first line\n  second line\n}";
        let result = tokenizer.tokenize(input).expect("Tokenization failed");

        // Expected tokens: CodeBlock
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token_type, "CodeBlock");
        assert_eq!(result[0].value, "{\n  first line\n  second line\n}");

        // Verify block contains newlines and whitespace
        assert!(result[0].value.contains("\n"));
        assert!(result[0].value.contains("  first"));
    }

    #[test]
    fn test_blocks_with_excluded_delimiters() {
        // Create a custom tokenizer that excludes delimiters
        let config = TokenizerConfig {
            tokenize_whitespace: true,
            continue_on_error: true,
            error_tolerance_limit: 5,
            track_token_positions: true,
        };
        let mut tokenizer = Tokenizer::with_config(config);

        // Add block scanner that excludes delimiters
        tokenizer.add_block_scanner(
            "{",
            "}",
            "CodeBlock",
            Some("WithoutDelimiters"),
            true,
            false,
            false // Exclude delimiters
        );

        let input = "{ code block content }";
        let result = tokenizer.tokenize(input).expect("Tokenization failed");

        // Expected tokens: CodeBlock
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token_type, "CodeBlock");
        assert_eq!(result[0].token_sub_type, Some("WithoutDelimiters".to_string()));
        assert_eq!(result[0].value, " code block content ");

        // Verify the delimiters are excluded
        assert!(!result[0].value.contains("{"));
        assert!(!result[0].value.contains("}"));
    }
}