//! Tests specifically targeting uncovered code paths in json_helper.rs

use serde::{Serialize, Deserialize};
use superconfig_macros::generate_json_helper;

#[test] 
fn test_empty_attribute_parsing() {
    // This should trigger the empty input break condition in attribute parsing
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        value: i32,
    }
    
    impl TestStruct {
        // Test with completely empty attribute - should use defaults
        #[generate_json_helper()]
        pub fn test_empty_attrs(self) -> Result<Self, String> {
            Ok(self)
        }
    }
    
    let test = TestStruct { value: 42 };
    let result = test.test_empty_attrs_as_json();
    assert!(result.contains("success"));
}

#[test]
fn test_complex_type_detection_edge_cases() {
    // Testing edge cases to hit uncovered lines in is_complex_type function
    
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        value: i32,
    }
    
    impl TestStruct {
        // Test with tuple type (line 128: _ => true branch)
        #[generate_json_helper]
        pub fn test_tuple_param(self, _data: (String, i32)) -> Result<Self, String> {
            Ok(self)
        }
        
        // Test with array type (line 128: _ => true branch)  
        #[generate_json_helper]
        pub fn test_array_param(self, _data: [i32; 3]) -> Result<Self, String> {
            Ok(self)
        }
    }
    
    let test = TestStruct { value: 42 };
    let result = test.test_tuple_param_json(r#"["hello", 123]"#);
    assert!(result.contains("success"));
    
    let test2 = TestStruct { value: 42 };
    let result2 = test2.test_array_param_json(r#"[1, 2, 3]"#);
    assert!(result2.contains("success"));
}

#[test]
fn test_arc_type_detection_edge_cases() {
    // Test edge cases in is_arc_type to hit lines 88, 91
    
    #[derive(Serialize, Clone)]
    struct TestStruct {
        value: i32,
    }
    
    impl TestStruct {
        // Test with deeply nested Arc path to trigger line 88 (false branch)
        #[generate_json_helper]
        pub fn test_complex_arc_path(self: std::sync::Arc<Self>) -> Result<Self, String> {
            Ok((*self).clone())
        }
    }
    
    let test = TestStruct { value: 42 };
    let arc_test = std::sync::Arc::new(test);
    let result = arc_test.test_complex_arc_path_as_json();
    assert!(result.contains("success"));
}

#[test]  
fn test_reference_type_edge_cases() {
    // Test reference types to hit lines 119, 122, 125
    
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        value: String,
    }
    
    impl TestStruct {
        // Test with &str reference (line 118-119: simple case)
        #[generate_json_helper]
        pub fn test_str_ref(self, _data: &str) -> Result<Self, String> {
            Ok(self)
        }
        
        // Test with complex reference type (line 119: true branch)
        #[generate_json_helper] 
        pub fn test_complex_ref(self, _data: Vec<String>) -> Result<Self, String> {
            let _ = _data; // Use the parameter
            Ok(self)
        }
    }
    
    let test = TestStruct { value: "test".to_string() };
    let result = test.test_str_ref_as_json(r#""hello""#);
    assert!(result.contains("success"));
    
    let test2 = TestStruct { value: "test".to_string() };
    let result2 = test2.test_complex_ref_json(r#"["item1", "item2"]"#);
    assert!(result2.contains("success"));
}

#[test]
fn test_type_path_without_segments() {
    // Test to hit line 109: type path without segments
    
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        value: i32,
    }
    
    impl TestStruct {
        // Test with generic type parameter that might not have segments
        #[generate_json_helper]
        pub fn test_generic(self, _data: Option<i32>) -> Result<Self, String> {
            Ok(self)
        }
    }
    
    let test = TestStruct { value: 42 };
    let result = test.test_generic_json(r#"42"#);
    assert!(result.contains("success"));
}

#[test]
fn test_auto_detect_directions_complex_params() {
    // Test auto_detect_directions to hit closure at line 134
    
    #[derive(Serialize, Deserialize)]
    struct ComplexParam {
        nested: Vec<String>,
    }
    
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        value: i32,
    }
    
    impl TestStruct {
        // Method with complex parameter should auto-detect incoming direction
        #[generate_json_helper]
        pub fn test_auto_detect_incoming(self, complex: ComplexParam) -> Result<Self, String> {
            let _ = complex; // Use the parameter
            Ok(self)
        }
    }
    
    let test = TestStruct { value: 42 };
    let result = test.test_auto_detect_incoming_json(r#"{"nested": ["item1", "item2"]}"#);
    assert!(result.contains("success"));
}