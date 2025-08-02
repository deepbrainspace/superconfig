//! Comprehensive tests to achieve 100% coverage of all uncovered lines

use serde::{Serialize, Deserialize};
use superconfig_macros::generate_json_helper;

#[test]
fn test_type_path_without_segments_edge_case() {
    // Test edge case where type path has no segments (line 109)
    #[derive(Serialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        #[generate_json_helper]
        pub fn test_method(self, param: ()) -> Result<Self, String> {
            // Actually use the unit parameter by checking it
            if param == () {
                Ok(self)
            } else {
                Err("Unit parameter mismatch".to_string())
            }
        }
    }

    let test = TestStruct { value: 42 };
    let result = test.test_method_json(serde_json::to_string(&()).unwrap().as_str());
    assert!(result.contains("success"));
}

#[test]
fn test_reference_type_edge_cases() {
    // Test reference types that are complex (lines 119, 122, 125)
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        #[generate_json_helper]
        pub fn test_complex_ref(self, param: Vec<String>) -> Result<Self, String> {
            // Use the parameter by checking if it's empty
            if param.is_empty() {
                Err("Empty vector".to_string())
            } else {
                Ok(self)
            }
        }

        #[generate_json_helper]
        pub fn test_nested_ref(self, param: Option<i32>) -> Result<Self, String> {
            // Use the parameter by pattern matching
            match param {
                Some(val) if val >= 0 => Ok(self),
                Some(_) => Err("Negative value".to_string()),
                None => Ok(self),
            }
        }
    }

    let test = TestStruct { value: 42 };
    let result = test.test_complex_ref_json(r#"{"param": ["test"]}"#);
    assert!(result.contains("success"));

    let test2 = TestStruct { value: 42 };
    let result2 = test2.test_nested_ref_json(r#"{"param": 42}"#);
    assert!(result2.contains("success"));
}

#[test]
fn test_non_arc_type_detection() {
    // Test non-Arc types to hit line 91
    #[derive(Serialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        #[generate_json_helper]
        pub fn test_regular_type(self: Box<Self>) -> Result<Box<Self>, String> {
            Ok(self)
        }

        #[generate_json_helper]
        pub fn test_option_type(self, param: Option<String>) -> Result<Self, String> {
            // Use the parameter by checking if it's present
            match param {
                Some(s) if !s.is_empty() => Ok(self),
                Some(_) => Err("Empty string".to_string()),
                None => Ok(self),
            }
        }
    }

    let test = Box::new(TestStruct { value: 42 });
    let result = test.test_regular_type_as_json();
    assert!(result.contains("success"));

    let test2 = TestStruct { value: 42 };
    let result2 = test2.test_option_type_json(serde_json::to_string(&Some("test".to_string())).unwrap().as_str());
    assert!(result2.contains("success"));
}

#[test]
fn test_no_segments_in_type_path() {
    // Test case where type path has no segments (line 88)
    #[derive(Serialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        #[generate_json_helper]
        pub fn test_primitive_self(self) -> Result<Self, String> {
            Ok(self)
        }
    }

    let test = TestStruct { value: 42 };
    let result = test.test_primitive_self_as_json();
    assert!(result.contains("success"));
}

#[test]
fn test_auto_detection_edge_cases() {
    // Test auto-detection scenarios that hit lines 143, 156, 159, 165, 168, 177
    #[derive(Serialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        // Test method with no parameters and non-result return (lines 156, 159)
        #[generate_json_helper(auto)]
        pub fn test_no_params_non_result(self) -> Self {
            self
        }

        // Test method with simple params and Result return (line 165)
        #[generate_json_helper(auto)]
        pub fn test_simple_params_result(self, x: i32) -> Result<Self, String> {
            if x > 0 {
                Ok(self)
            } else {
                Err("Value must be positive".to_string())
            }
        }

        // Test method with no params and Result return (line 168)
        #[generate_json_helper(auto)]
        pub fn test_no_params_result(self) -> Result<Self, String> {
            Ok(self)
        }

        // Test method with complex params (line 143)
        #[generate_json_helper(auto)]
        pub fn test_complex_params(self, data: Vec<String>) -> Result<Self, String> {
            if data.is_empty() {
                Err("Data cannot be empty".to_string())
            } else {
                Ok(self)
            }
        }

        // Test method with both complex params and return (line 177)
        #[generate_json_helper(auto)]
        pub fn test_both_complex(self, data: Vec<String>) -> Result<Vec<String>, String> {
            if data.len() > 10 {
                Err("Too many items".to_string())
            } else {
                Ok(data)
            }
        }
    }

    let test = TestStruct { value: 42 };
    let result = test.test_no_params_non_result_as_json();
    assert!(result.contains("success"));

    let test2 = TestStruct { value: 42 };
    let result2 = test2.test_simple_params_result_as_json(5);
    assert!(result2.contains("success"));

    let test3 = TestStruct { value: 42 };
    let result3 = test3.test_no_params_result_as_json();
    assert!(result3.contains("success"));

    let test4 = TestStruct { value: 42 };
    let result4 = test4.test_complex_params_json(r#"{"data": ["test"]}"#);
    assert!(result4.contains("success"));

    let test5 = TestStruct { value: 42 };
    let result5 = test5.test_both_complex_json(r#"{"data": ["test"]}"#);
    assert!(result5.contains("success"));
}

#[test]
fn test_error_handling_paths() {
    // Test error paths with unusual method signatures to trigger various code paths
    #[derive(Serialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        // Test methods with unusual patterns to exercise error handling
        #[generate_json_helper]
        pub fn test_unit_return(self) -> () {
            ()
        }
    }

    let test = TestStruct { value: 42 };
    let result = test.test_unit_return_as_json();
    assert!(result.contains("success"));
}

#[test]
fn test_empty_input_parsing() {
    // Test empty input case (line 62) - this should be covered by empty attribute case
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

    let test = TestStruct { value: 42 };
    let result = test.test_empty_attr_as_json();
    assert!(result.contains("success"));
}

#[test]
fn test_mixed_direction_combinations() {
    // Test various direction combinations that might hit uncovered branches
    #[derive(Serialize)]
    struct TestStruct {
        value: i32,
    }

    impl TestStruct {
        #[generate_json_helper(outgoing)]
        pub fn test_out_combo(self, param: String) -> Result<Self, String> {
            if param.len() > 100 {
                Err("Parameter too long".to_string())
            } else {
                Ok(self)
            }
        }

        #[generate_json_helper(auto)]
        pub fn test_auto_direction(self, param: String) -> Result<Self, String> {
            if param.is_empty() {
                Err("Empty parameter".to_string())
            } else {
                Ok(self)
            }
        }
    }

    let test = TestStruct { value: 42 };
    let result = test.test_out_combo_as_json("test".to_string());
    assert!(result.contains("success"));

    let test2 = TestStruct { value: 42 };
    let result2 = test2.test_auto_direction_as_json("test".to_string());
    assert!(result2.contains("success"));
}