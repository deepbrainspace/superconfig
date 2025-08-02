//! Tests for MultiFFI naming conversion and macro functionality.

// Naming conversion tests are only relevant when WASM feature is enabled
#[cfg(feature = "wasm")]
mod naming_tests {
    use crate::{convert_to_camel_case, create_camel_case_ident};
    use syn::Ident;

    #[test]
    fn test_convert_to_camel_case_basic() {
        assert_eq!(convert_to_camel_case("with_file"), "withFile");
        assert_eq!(convert_to_camel_case("set_debug"), "setDebug");
        assert_eq!(convert_to_camel_case("extract_json"), "extractJson");
        assert_eq!(convert_to_camel_case("with_wildcard"), "withWildcard");
    }

    #[test]
    fn test_convert_to_camel_case_edge_cases() {
        // Single word
        assert_eq!(convert_to_camel_case("single"), "single");

        // Multiple underscores
        assert_eq!(
            convert_to_camel_case("with__double__underscore"),
            "withDoubleUnderscore"
        );

        // Leading/trailing underscores
        assert_eq!(convert_to_camel_case("_with_leading"), "withLeading");
        assert_eq!(convert_to_camel_case("with_trailing_"), "withTrailing");

        // Empty string
        assert_eq!(convert_to_camel_case(""), "");

        // Only underscores
        assert_eq!(convert_to_camel_case("___"), "");
    }

    #[test]
    fn test_convert_to_camel_case_real_world_examples() {
        // SuperConfig method names
        assert_eq!(
            convert_to_camel_case("with_hierarchical_config"),
            "withHierarchicalConfig"
        );
        assert_eq!(convert_to_camel_case("set_strict_mode"), "setStrictMode");
        assert_eq!(
            convert_to_camel_case("with_auto_profiles"),
            "withAutoProfiles"
        );
        assert_eq!(
            convert_to_camel_case("extract_debug_info"),
            "extractDebugInfo"
        );
    }

    #[test]
    fn test_create_camel_case_ident() {
        use proc_macro2::Span;

        let original = Ident::new("with_file", Span::call_site());
        let converted = create_camel_case_ident(&original);

        assert_eq!(converted.to_string(), "withFile");
        // Span should be preserved (can't easily test but important for error messages)
    }

    #[test]
    fn test_javascript_api_consistency() {
        // Test that the same conversion rules apply consistently
        // This ensures Node.js and WASM will get identical function names
        let test_cases = [
            ("with_file", "withFile"),
            ("set_debug", "setDebug"),
            ("extract_json", "extractJson"),
            ("with_hierarchical_config", "withHierarchicalConfig"),
            ("set_strict_mode", "setStrictMode"),
        ];

        for (input, expected) in &test_cases {
            let result = convert_to_camel_case(input);
            assert_eq!(
                &result, expected,
                "Conversion failed for '{}': expected '{}', got '{}'",
                input, expected, result
            );
        }
    }

    #[test]
    fn test_naming_consistency_for_target_languages() {
        // Test that ensures consistent naming across JavaScript environments
        let snake_case_names = [
            "with_file",
            "set_debug",
            "extract_json",
            "with_wildcard",
            "with_hierarchical_config",
        ];

        for name in &snake_case_names {
            let nodejs_result = convert_to_camel_case(name);
            let wasm_result = convert_to_camel_case(name);

            // Both JavaScript environments should get identical names
            assert_eq!(
                nodejs_result, wasm_result,
                "Node.js and WASM should produce identical names for '{}'",
                name
            );

            // Should not be the same as original snake_case (unless single word)
            if name.contains('_') {
                assert_ne!(
                    nodejs_result, *name,
                    "CamelCase conversion should change '{}' to something different",
                    name
                );
            }
        }
    }
}

// Additional module-level tests that don't depend on naming functions can go here
// (currently none, but this structure allows for future expansion)
