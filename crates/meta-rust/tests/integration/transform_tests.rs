use meta_rust::for_each;

#[test]
fn test_for_each_with_snake_transform() {
    // Test snake_case transformation in for_each
    for_each!([getUserName, updateUserProfile], |method| {
        fn %{method:snake}_handler() -> &'static str {
            "%{method:snake}"
        }
    });

    assert_eq!(get_user_name_handler(), "get_user_name");
    assert_eq!(update_user_profile_handler(), "update_user_profile");
}

#[test]
fn test_for_each_with_upper_transform() {
    // Test uppercase transformation
    for_each!([get, post, put], |method| {
        const %{method:upper}_METHOD: &str = "%{method:upper}";
    });

    assert_eq!(GET_METHOD, "GET");
    assert_eq!(POST_METHOD, "POST");
    assert_eq!(PUT_METHOD, "PUT");
}

#[test]
fn test_for_each_with_multiple_transforms() {
    // Test multiple transforms in same template
    for_each!([createUser, deletePost], |action| {
        fn %{action:snake}() -> (&'static str, &'static str) {
            ("%{action:kebab}", "%{action:pascal}")
        }
    });

    assert_eq!(create_user(), ("create-user", "CreateUser"));
    assert_eq!(delete_post(), ("delete-post", "DeletePost"));
}

#[test]
fn test_for_each_arrays_with_transforms() {
    // Test array items with transforms
    for_each!([["users", "getUser"], ["posts", "getPost"]], |resource| {
        fn %{resource[1]:snake}_from_%{resource[0]}() -> &'static str {
            "%{resource[0]:upper}"
        }
    });

    assert_eq!(get_user_from_users(), "USERS");
    assert_eq!(get_post_from_posts(), "POSTS");
}

#[test]
fn test_logffi_real_use_case() {
    // The actual LogFFI use case
    for_each!([error, warn, info], |level| {
        macro_rules! %{level:snake}_log {
            ($msg:expr) => {
                format!("[%{level:upper}] {}", $msg)
            };
        }
    });

    assert_eq!(error_log!("failed"), "[ERROR] failed");
    assert_eq!(warn_log!("warning"), "[WARN] warning");
    assert_eq!(info_log!("information"), "[INFO] information");
}

#[test]
fn test_all_case_transforms() {
    // Test all case transformation types
    for_each!([getUserData], |method| {
        const %{method:snake}_SNAKE: &str = "%{method:snake}";
        const %{method:camel}_CAMEL: &str = "%{method:camel}";
        const %{method:snake}_KEBAB: &str = "%{method:kebab}";
        const %{method:pascal}_PASCAL: &str = "%{method:pascal}";
        const %{method:snake}_TITLE: &str = "%{method:title}";
    });

    assert_eq!(get_user_data_SNAKE, "get_user_data");
    assert_eq!(getUserData_CAMEL, "getUserData");
    assert_eq!(get_user_data_KEBAB, "get-user-data");
    assert_eq!(GetUserData_PASCAL, "GetUserData");
    assert_eq!(get_user_data_TITLE, "Get User Data");
}

#[test]
fn test_other_transforms() {
    // Test lower, reverse, len transforms
    for_each!([HELLO], |text| {
        const %{text:lower}_LOWER: &str = "%{text:lower}";
        const %{text}_REVERSE: &str = "%{text:reverse}";
        const %{text}_LEN: &str = "%{text:len}";
    });

    assert_eq!(hello_LOWER, "hello");
    assert_eq!(HELLO_REVERSE, "OLLEH");
    assert_eq!(HELLO_LEN, "5");
}

#[test]
fn test_transforms_with_string_literals() {
    // Test transforms work with string literals
    for_each!(["getUserName", "updateUserProfile"], |method| {
        fn %{method:snake}_string() -> &'static str {
            "%{method:pascal}"
        }
    });

    assert_eq!(get_user_name_string(), "GetUserName");
    assert_eq!(update_user_profile_string(), "UpdateUserProfile");
}

#[test]
fn test_no_transform_passthrough() {
    // Test that %{param} without transform works
    for_each!([foo, bar], |item| {
        const %{item}_PLAIN: &str = "%{item}";
    });

    assert_eq!(foo_PLAIN, "foo");
    assert_eq!(bar_PLAIN, "bar");
}
