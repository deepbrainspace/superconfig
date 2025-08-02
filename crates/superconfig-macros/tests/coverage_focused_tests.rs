//! Simplified tests to exercise all macro code paths for coverage

use serde::Serialize;
use superconfig_macros::generate_json_helper;

#[derive(Serialize, Clone)]
struct SimpleStruct {
    value: i32,
}

impl SimpleStruct {
    // Test incoming direction
    #[generate_json_helper(incoming)]
    pub fn test_incoming(self, param: i32) -> Result<Self, String> {
        Ok(Self { value: param })
    }

    // Test outgoing direction
    #[generate_json_helper(outgoing)]
    pub fn test_outgoing(self) -> Result<Self, String> {
        Ok(self)
    }

    // Test auto detection
    #[generate_json_helper(auto)]
    pub fn test_auto(self, param: i32) -> Result<Self, String> {
        Ok(Self { value: param })
    }

    // Test bidirectional (legacy)
    #[generate_json_helper(bidirectional)]
    pub fn test_bidirectional(self, param: i32) -> Result<Self, String> {
        Ok(Self { value: param })
    }

    // Test handle_mode
    #[generate_json_helper(handle_mode)]
    pub fn test_handle_mode(self, param: i32) -> Result<Self, String> {
        Ok(Self { value: param })
    }

    // Test Arc detection
    #[generate_json_helper]
    pub fn test_arc(self: std::sync::Arc<Self>) -> Result<std::sync::Arc<Self>, String> {
        Ok(self)
    }

    // Test non-Arc
    #[generate_json_helper]
    pub fn test_normal(self) -> Result<Self, String> {
        Ok(self)
    }

    // Test non-Result return
    #[generate_json_helper]
    pub fn test_non_result(self) -> Self {
        self
    }

    // Test complex params
    #[generate_json_helper]
    pub fn test_complex(self, a: i32, b: String, c: Option<i32>) -> Result<Self, String> {
        println!("Complex test with string: {}", b);
        Ok(Self {
            value: a + c.unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_generation() {
        let s = SimpleStruct { value: 42 };

        // Just test that the generated methods exist by calling them
        // This will exercise the macro code during compilation
        let _result1 = s.clone().test_incoming_from_json(100, r#"{"param": 100}"#);
        let _result2 = s.clone().test_outgoing_as_json();
        let _result3 = s.clone().test_auto_as_json(200);
        let _result4 = s.clone().test_bidirectional_json(300, r#"{"param": 300}"#);
        let _result5 = s.clone().test_handle_mode(400);

        let arc_s = std::sync::Arc::new(s.clone());
        let _result6 = arc_s.test_arc_as_json();

        let _result7 = s.clone().test_normal_as_json();
        let _result8 = s.clone().test_non_result_as_json();
        let _result9 = s.test_complex_json(500, "test".to_string(), r#"{"a": 500, "b": "test", "c": 50}"#);
    }
}
