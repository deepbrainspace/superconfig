//! Tests for unified JSON helper functionality (bidirectional in single method)

use serde::{Deserialize, Serialize};
use superconfig_macros::generate_json_helper;

// Complex types for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskConfig {
    pub name: String,
    pub priority: u8,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskResult {
    pub id: u64,
    pub status: String,
    pub config: TaskConfig,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessingError {
    pub message: String,
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProcessingError: {}", self.message)
    }
}

impl std::error::Error for ProcessingError {}

// Service with unified JSON methods
#[derive(Debug, Clone, Serialize)]
pub struct TaskProcessor {
    pub processed_count: u32,
}

impl TaskProcessor {
    pub fn new() -> Self {
        Self { processed_count: 0 }
    }

    // This method has complex input AND complex output
    // Auto mode should generate a UNIFIED method: process_task_json()
    #[generate_json_helper(auto)]
    pub fn process_task(self, task_config: TaskConfig) -> Result<TaskResult, ProcessingError> {
        if task_config.name.is_empty() {
            return Err(ProcessingError {
                message: "Task name cannot be empty".to_string(),
            });
        }

        if task_config.priority > 5 {
            return Err(ProcessingError {
                message: "Priority cannot exceed 5".to_string(),
            });
        }

        // Note: can't modify self since we moved it, but that's OK for this example

        Ok(TaskResult {
            id: self.processed_count as u64,
            status: "completed".to_string(),
            config: TaskConfig {
                name: format!("processed_{}", task_config.name),
                priority: task_config.priority,
                tags: task_config.tags,
            },
            duration_ms: 150,
        })
    }

    // Simple input + complex output = should generate _as_json only
    #[generate_json_helper(auto)]
    pub fn get_summary(self, include_details: bool) -> Result<TaskResult, ProcessingError> {
        if !include_details {
            return Err(ProcessingError {
                message: "Details are required".to_string(),
            });
        }

        Ok(TaskResult {
            id: 999,
            status: "summary".to_string(),
            config: TaskConfig {
                name: "summary_task".to_string(),
                priority: 1,
                tags: vec!["summary".to_string()],
            },
            duration_ms: 50,
        })
    }

    // Complex input + simple output = should generate _from_json only
    #[generate_json_helper(auto)]
    pub fn validate_config(self, config: TaskConfig) -> Result<Self, ProcessingError> {
        if config.name.len() < 3 {
            return Err(ProcessingError {
                message: "Config name too short".to_string(),
            });
        }

        Ok(self)
    }

    // Explicit unified method
    #[generate_json_helper(bidirectional)]
    pub fn transform_task(self, input: TaskConfig) -> Result<TaskResult, ProcessingError> {
        Ok(TaskResult {
            id: 42,
            status: "transformed".to_string(),
            config: input,
            duration_ms: 100,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_json_method_exists() {
        let processor = TaskProcessor::new();

        // Auto mode detected complex input + complex output = unified method
        let json_input = r#"{
            "task_config": {
                "name": "test_task",
                "priority": 3,
                "tags": ["urgent", "backend"]
            }
        }"#;

        let result = processor.process_task_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert!(parsed["data"].is_object());

        let data = &parsed["data"];
        assert_eq!(data["status"], "completed");
        assert_eq!(data["config"]["name"], "processed_test_task");
        assert_eq!(data["duration_ms"], 150);
    }

    #[test]
    fn test_unified_json_validation_error() {
        let processor = TaskProcessor::new();

        // Invalid priority
        let json_input = r#"{
            "task_config": {
                "name": "test_task",
                "priority": 10,
                "tags": ["invalid"]
            }
        }"#;

        let result = processor.process_task_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Priority cannot exceed 5")
        );
    }

    #[test]
    fn test_unified_json_deserialization_error() {
        let processor = TaskProcessor::new();

        // Invalid JSON structure
        let json_input = r#"{
            "task_config": {
                "name": "test_task",
                "priority": "invalid_number",
                "tags": ["test"]
            }
        }"#;

        let result = processor.process_task_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Failed to deserialize parameter")
        );
    }

    #[test]
    fn test_unified_json_missing_parameter() {
        let processor = TaskProcessor::new();

        // Missing task_config field
        let json_input = r#"{
            "other_field": "irrelevant"
        }"#;

        let result = processor.process_task_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Missing required parameter")
        );
        assert!(parsed["error"].as_str().unwrap().contains("task_config"));
    }

    #[test]
    fn test_auto_detects_as_json_only() {
        let processor = TaskProcessor::new();

        // Simple input + complex output = should have _as_json method only
        let result = processor.get_summary_as_json(true);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["status"], "summary");

        // Should NOT have _from_json method (this would cause compile error)
        // processor.get_summary_from_json("{}"); // This should not exist
    }

    #[test]
    fn test_auto_detects_unified_method() {
        let processor = TaskProcessor::new();

        // Complex input + Self output = auto-detects as bidirectional (unified method)
        let json_input = r#"{
            "config": {
                "name": "valid_config",
                "priority": 2,
                "tags": ["test"]
            }
        }"#;

        let result = processor.validate_config_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);

        // This is actually generating a unified method because Self is considered complex
        // This is correct behavior for builder patterns!
    }

    #[test]
    fn test_explicit_bidirectional() {
        let processor = TaskProcessor::new();

        // Explicitly marked as bidirectional = should have unified _json method
        let json_input = r#"{
            "input": {
                "name": "transform_me",
                "priority": 1,
                "tags": ["transform"]
            }
        }"#;

        let result = processor.transform_task_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["status"], "transformed");
        assert_eq!(parsed["data"]["config"]["name"], "transform_me");
    }

    #[test]
    fn test_original_methods_still_work() {
        let processor = TaskProcessor::new();

        let config = TaskConfig {
            name: "original_test".to_string(),
            priority: 2,
            tags: vec!["original".to_string()],
        };

        // Original method should still work
        let result = processor.process_task(config).unwrap();
        assert_eq!(result.config.name, "processed_original_test");
        assert_eq!(result.status, "completed");
    }

    #[test]
    fn test_json_response_structure() {
        let processor = TaskProcessor::new();

        let json_input = r#"{
            "task_config": {
                "name": "structure_test",
                "priority": 1,
                "tags": ["test"]
            }
        }"#;

        let result = processor.process_task_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Check response structure
        assert!(parsed.is_object());
        assert!(parsed.get("success").is_some());

        if parsed["success"] == true {
            assert!(parsed.get("data").is_some());
            assert!(parsed["data"].is_object());
        } else {
            assert!(parsed.get("error").is_some());
            assert!(parsed["error"].is_string());
        }
    }
}
