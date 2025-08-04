use meta_rust::for_each;

#[test]
fn test_single_items_basic() {
    // Test that single items work - this should generate 3 functions
    for_each!([error, warn, info], |level| {
        pub fn test_%{level}() -> &'static str {
            stringify!(%{level})
        }
    });

    // Test the generated functions
    assert_eq!(test_error(), "error");
    assert_eq!(test_warn(), "warn");
    assert_eq!(test_info(), "info");
}

#[test]
fn test_string_items() {
    // Test that string items work
    for_each!(["GET", "POST", "PUT"], |method| {
        pub fn handle_%{method}() -> &'static str {
            "%{method}"
        }
    });

    // Test the generated functions
    assert_eq!(handle_GET(), "GET");
    assert_eq!(handle_POST(), "POST");
    assert_eq!(handle_PUT(), "PUT");
}

#[test]
fn test_array_items() {
    // Test that array items work with indexing
    for_each!([["GET", 200], ["POST", 201]], |req| {
        pub fn status_%{req[0]}() -> u16 {
            %{req[1]}
        }
    });

    // Test the generated functions
    assert_eq!(status_GET(), 200);
    assert_eq!(status_POST(), 201);
}

#[test]
fn test_number_items() {
    // Test that number items work
    for_each!([200, 404, 500], |code| {
        pub fn status_%{code}() -> u16 {
            %{code}
        }
    });

    // Test the generated functions
    assert_eq!(status_200(), 200);
    assert_eq!(status_404(), 404);
    assert_eq!(status_500(), 500);
}

#[test]
fn test_mixed_types() {
    // Test that mixed types work in same array
    for_each!([error, "GET", 200], |item| {
        pub fn mixed_%{item}() -> &'static str {
            stringify!(%{item})
        }
    });

    // Test the generated functions
    assert_eq!(mixed_error(), "error");
    assert_eq!(mixed_GET(), "GET");
    assert_eq!(mixed_200(), "200");
}

#[test]
fn test_multiple_parameter_references() {
    // Test multiple %{param} references in same template
    for_each!([debug, info], |level| {
        pub fn %{level}_log_%{level}() -> &'static str {
            concat!(stringify!(%{level}), "_", stringify!(%{level}))
        }
    });

    // Test the generated functions
    assert_eq!(debug_log_debug(), "debug_debug");
    assert_eq!(info_log_info(), "info_info");
}

#[test]
fn test_complex_array_indexing() {
    // Test multi-element arrays with multiple index references
    for_each!([["users", "GET", "/api/users"], ["posts", "POST", "/api/posts"]], |route| {
        pub fn %{route[0]}_%{route[1]}() -> &'static str {
            "%{route[2]}"
        }
    });

    // Test the generated functions
    assert_eq!(users_GET(), "/api/users");
    assert_eq!(posts_POST(), "/api/posts");
}

#[test]
fn test_logffi_use_case() {
    // Test the actual LogFFI use case - generating macros
    for_each!([error, warn, info], |level| {
        macro_rules! %{level}_log {
            ($msg:expr) => {
                format!("[{}] {}", stringify!(%{level}).to_uppercase(), $msg)
            };
        }
    });

    // Test the generated macros
    assert_eq!(error_log!("failed"), "[ERROR] failed");
    assert_eq!(warn_log!("warning"), "[WARN] warning");
    assert_eq!(info_log!("started"), "[INFO] started");
}

#[test]
fn test_advanced_macro_generation() {
    // Test generating macros with different patterns - simpler example
    for_each!([["create", "user"], ["delete", "post"]], |action| {
        macro_rules! %{action[0]}_%{action[1]}_macro {
            ($id:expr) => {
                format!("{}_{}_action: {}",
                    stringify!(%{action[0]}),
                    stringify!(%{action[1]}),
                    $id)
            };
        }
    });

    // Test the generated macros
    assert_eq!(create_user_macro!(123), "create_user_action: 123");
    assert_eq!(delete_post_macro!(456), "delete_post_action: 456");
}
