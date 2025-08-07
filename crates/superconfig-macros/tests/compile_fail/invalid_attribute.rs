use serde::Serialize;
use superconfig_macros::generate_json_helper;

#[derive(Serialize)]
struct TestStruct {
    value: i32,
}

impl TestStruct {
    #[generate_json_helper(invalid_attribute)]
    pub fn test_method(self) -> Result<Self, String> {
        Ok(self)
    }
}

fn main() {}