//! Tests for error handling in the `#[generate_json_helper]` procedural macro

#[test]
fn test_compile_fail_cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}

#[test]
fn test_arc_detection_path() {
    // Test the Arc detection functionality by creating a macro that uses Arc
    use serde::Serialize;
    use superconfig_macros::generate_json_helper;

    #[derive(Serialize, Clone)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        #[generate_json_helper]
        pub fn test_arc_method(self: std::sync::Arc<Self>) -> Result<std::sync::Arc<Self>, String> {
            Ok(self)
        }

        #[generate_json_helper]
        pub fn test_non_arc_method(mut self) -> Result<Self, String> {
            self.value += 1;
            Ok(self)
        }
    }

    // Test that both methods compile and work
    let test_struct = TestStruct { value: 42 };
    let arc_struct = std::sync::Arc::new(test_struct);
    let result = arc_struct.test_arc_method_as_json();
    assert!(result.contains("success"));

    let test_struct2 = TestStruct { value: 24 };
    let result2 = test_struct2.test_non_arc_method_as_json();
    assert!(result2.contains("success"));
}

#[test]
fn test_empty_attribute_parsing() {
    // Test empty attribute list parsing
    use serde::Serialize;
    use superconfig_macros::generate_json_helper;

    #[derive(Serialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        #[generate_json_helper()]
        pub fn test_empty_attr(self) -> Result<Self, String> {
            Ok(self)
        }
    }

    // Test that it compiles and works
    let test = TestStruct { value: 100 };
    let result = test.test_empty_attr_as_json();
    assert!(result.contains("success"));
}
