use anyhow::Result;
use depyler_annotations::LambdaEventType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lambda-specific type mappings and conversions
#[derive(Debug, Clone)]
pub struct LambdaTypeMapper {
    event_mappings: HashMap<LambdaEventType, EventTypeMapping>,
    response_mappings: HashMap<LambdaEventType, ResponseTypeMapping>,
}

#[derive(Debug, Clone)]
pub struct EventTypeMapping {
    pub rust_type: String,
    pub aws_events_module: String,
    pub imports: Vec<String>,
    pub serde_attributes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResponseTypeMapping {
    pub rust_type: String,
    pub conversion_impl: Option<String>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConversionRule {
    pub python_pattern: String,
    pub rust_type: String,
    pub lambda_context: LambdaContext,
    pub serde_attribute: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LambdaContext {
    EventRoot,
    HeaderValue,
    StatusCode,
    Timestamp,
    Records,
    RequestContext,
    PathParameters,
    QueryStringParameters,
    Body,
}

impl Default for LambdaTypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl LambdaTypeMapper {
    pub fn new() -> Self {
        let mut event_mappings = HashMap::new();
        let mut response_mappings = HashMap::new();

        // S3 Event mappings
        event_mappings.insert(
            LambdaEventType::S3Event,
            EventTypeMapping {
                rust_type: "S3Event".to_string(),
                aws_events_module: "s3".to_string(),
                imports: vec![
                    "use aws_lambda_events::s3::S3Event;".to_string(),
                ],
                serde_attributes: vec![],
            },
        );

        response_mappings.insert(
            LambdaEventType::S3Event,
            ResponseTypeMapping {
                rust_type: "serde_json::Value".to_string(),
                conversion_impl: None,
                imports: vec!["use serde_json;".to_string()],
            },
        );

        // API Gateway v1 mappings
        event_mappings.insert(
            LambdaEventType::ApiGatewayProxyRequest,
            EventTypeMapping {
                rust_type: "ApiGatewayProxyRequest".to_string(),
                aws_events_module: "apigw".to_string(),
                imports: vec![
                    "use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};".to_string(),
                    "use std::collections::HashMap;".to_string(),
                ],
                serde_attributes: vec![],
            },
        );

        response_mappings.insert(
            LambdaEventType::ApiGatewayProxyRequest,
            ResponseTypeMapping {
                rust_type: "ApiGatewayProxyResponse".to_string(),
                conversion_impl: Some(APIGW_RESPONSE_IMPL.to_string()),
                imports: vec![
                    "use aws_lambda_events::apigw::ApiGatewayProxyResponse;".to_string(),
                    "use std::collections::HashMap;".to_string(),
                ],
            },
        );

        // API Gateway v2 mappings
        event_mappings.insert(
            LambdaEventType::ApiGatewayV2HttpRequest,
            EventTypeMapping {
                rust_type: "ApiGatewayV2httpRequest".to_string(),
                aws_events_module: "apigw".to_string(),
                imports: vec![
                    "use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};".to_string(),
                    "use std::collections::HashMap;".to_string(),
                ],
                serde_attributes: vec![],
            },
        );

        response_mappings.insert(
            LambdaEventType::ApiGatewayV2HttpRequest,
            ResponseTypeMapping {
                rust_type: "ApiGatewayV2httpResponse".to_string(),
                conversion_impl: Some(APIGW_V2_RESPONSE_IMPL.to_string()),
                imports: vec![
                    "use aws_lambda_events::apigw::ApiGatewayV2httpResponse;".to_string(),
                    "use std::collections::HashMap;".to_string(),
                ],
            },
        );

        // SQS Event mappings
        event_mappings.insert(
            LambdaEventType::SqsEvent,
            EventTypeMapping {
                rust_type: "SqsEvent".to_string(),
                aws_events_module: "sqs".to_string(),
                imports: vec![
                    "use aws_lambda_events::sqs::{SqsEvent, SqsBatchResponse, SqsBatchItemFailure};".to_string(),
                ],
                serde_attributes: vec![],
            },
        );

        response_mappings.insert(
            LambdaEventType::SqsEvent,
            ResponseTypeMapping {
                rust_type: "SqsBatchResponse".to_string(),
                conversion_impl: None,
                imports: vec![
                    "use aws_lambda_events::sqs::SqsBatchResponse;".to_string(),
                ],
            },
        );

        // SNS Event mappings
        event_mappings.insert(
            LambdaEventType::SnsEvent,
            EventTypeMapping {
                rust_type: "SnsEvent".to_string(),
                aws_events_module: "sns".to_string(),
                imports: vec![
                    "use aws_lambda_events::sns::SnsEvent;".to_string(),
                ],
                serde_attributes: vec![],
            },
        );

        response_mappings.insert(
            LambdaEventType::SnsEvent,
            ResponseTypeMapping {
                rust_type: "serde_json::Value".to_string(),
                conversion_impl: None,
                imports: vec!["use serde_json;".to_string()],
            },
        );

        // DynamoDB Event mappings
        event_mappings.insert(
            LambdaEventType::DynamodbEvent,
            EventTypeMapping {
                rust_type: "DynamodbEvent".to_string(),
                aws_events_module: "dynamodb".to_string(),
                imports: vec![
                    "use aws_lambda_events::dynamodb::DynamodbEvent;".to_string(),
                ],
                serde_attributes: vec![],
            },
        );

        response_mappings.insert(
            LambdaEventType::DynamodbEvent,
            ResponseTypeMapping {
                rust_type: "serde_json::Value".to_string(),
                conversion_impl: None,
                imports: vec!["use serde_json;".to_string()],
            },
        );

        // EventBridge mappings
        event_mappings.insert(
            LambdaEventType::EventBridgeEvent(None),
            EventTypeMapping {
                rust_type: "EventBridgeEvent<serde_json::Value>".to_string(),
                aws_events_module: "eventbridge".to_string(),
                imports: vec![
                    "use aws_lambda_events::eventbridge::EventBridgeEvent;".to_string(),
                    "use serde_json;".to_string(),
                ],
                serde_attributes: vec![],
            },
        );

        response_mappings.insert(
            LambdaEventType::EventBridgeEvent(None),
            ResponseTypeMapping {
                rust_type: "()".to_string(),
                conversion_impl: None,
                imports: vec![],
            },
        );

        Self {
            event_mappings,
            response_mappings,
        }
    }

    /// Get event type mapping for a Lambda event type
    pub fn get_event_mapping(&self, event_type: &LambdaEventType) -> Option<&EventTypeMapping> {
        // Handle EventBridge with custom types
        if let LambdaEventType::EventBridgeEvent(Some(_)) = event_type {
            return self.event_mappings.get(&LambdaEventType::EventBridgeEvent(None));
        }
        
        self.event_mappings.get(event_type)
    }

    /// Get response type mapping for a Lambda event type
    pub fn get_response_mapping(&self, event_type: &LambdaEventType) -> Option<&ResponseTypeMapping> {
        // Handle EventBridge with custom types
        if let LambdaEventType::EventBridgeEvent(Some(_)) = event_type {
            return self.response_mappings.get(&LambdaEventType::EventBridgeEvent(None));
        }
        
        self.response_mappings.get(event_type)
    }

    /// Generate Python to Rust type conversion rules for Lambda context
    pub fn get_type_conversion_rules(&self) -> Vec<TypeConversionRule> {
        vec![
            TypeConversionRule {
                python_pattern: "dict".to_string(),
                rust_type: "T: Deserialize".to_string(),
                lambda_context: LambdaContext::EventRoot,
                serde_attribute: Some("#[serde(rename_all = \"camelCase\")]".to_string()),
            },
            TypeConversionRule {
                python_pattern: "str".to_string(),
                rust_type: "HeaderValue".to_string(),
                lambda_context: LambdaContext::HeaderValue,
                serde_attribute: Some("#[serde(with = \"http_serde::header_value\")]".to_string()),
            },
            TypeConversionRule {
                python_pattern: "int".to_string(),
                rust_type: "u16".to_string(),
                lambda_context: LambdaContext::StatusCode,
                serde_attribute: None,
            },
            TypeConversionRule {
                python_pattern: "float".to_string(),
                rust_type: "f64".to_string(),
                lambda_context: LambdaContext::Timestamp,
                serde_attribute: Some("#[serde(with = \"aws_lambda_events::time::float_unix_epoch\")]".to_string()),
            },
            TypeConversionRule {
                python_pattern: "List[dict]".to_string(),
                rust_type: "Vec<Record>".to_string(),
                lambda_context: LambdaContext::Records,
                serde_attribute: None,
            },
            TypeConversionRule {
                python_pattern: "Dict[str, str]".to_string(),
                rust_type: "HashMap<String, String>".to_string(),
                lambda_context: LambdaContext::PathParameters,
                serde_attribute: Some("#[serde(default)]".to_string()),
            },
            TypeConversionRule {
                python_pattern: "Dict[str, List[str]]".to_string(),
                rust_type: "HashMap<String, Vec<String>>".to_string(),
                lambda_context: LambdaContext::QueryStringParameters,
                serde_attribute: Some("#[serde(default)]".to_string()),
            },
            TypeConversionRule {
                python_pattern: "str".to_string(),
                rust_type: "Option<String>".to_string(),
                lambda_context: LambdaContext::Body,
                serde_attribute: None,
            },
        ]
    }

    /// Generate custom structs for EventBridge events with custom types
    pub fn generate_custom_eventbridge_types(&self, custom_type: &str) -> Result<String> {
        Ok(format!(
            r#"#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct {custom_type} {{
    // Define your custom event fields here
    // Example:
    // pub event_id: String,
    // pub timestamp: String,
    // pub data: serde_json::Value,
}}

#[derive(Debug, Deserialize)]
#[serde(tag = "detail-type", content = "detail")]
pub enum EventType {{
    #[serde(rename = "Custom Event")]
    CustomEvent({custom_type}),
    // Add more event variants as needed
}}
"#
        ))
    }

    /// Generate type-safe response builders
    pub fn generate_response_builders(&self, event_type: &LambdaEventType) -> Result<String> {
        match event_type {
            LambdaEventType::ApiGatewayProxyRequest => {
                Ok(APIGW_RESPONSE_BUILDER.to_string())
            }
            LambdaEventType::ApiGatewayV2HttpRequest => {
                Ok(APIGW_V2_RESPONSE_BUILDER.to_string())
            }
            LambdaEventType::SqsEvent => {
                Ok(SQS_RESPONSE_BUILDER.to_string())
            }
            _ => Ok(String::new()),
        }
    }

    /// Add custom event type mapping
    pub fn add_custom_event_mapping(&mut self, event_type: LambdaEventType, mapping: EventTypeMapping) {
        self.event_mappings.insert(event_type, mapping);
    }

    /// Add custom response type mapping
    pub fn add_custom_response_mapping(&mut self, event_type: LambdaEventType, mapping: ResponseTypeMapping) {
        self.response_mappings.insert(event_type, mapping);
    }

    /// Generate all required imports for an event type
    pub fn get_required_imports(&self, event_type: &LambdaEventType) -> Vec<String> {
        let mut imports = Vec::new();
        
        if let Some(event_mapping) = self.get_event_mapping(event_type) {
            imports.extend(event_mapping.imports.clone());
        }
        
        if let Some(response_mapping) = self.get_response_mapping(event_type) {
            imports.extend(response_mapping.imports.clone());
        }

        // Remove duplicates
        imports.sort();
        imports.dedup();
        imports
    }

    /// Generate error handling conversion for Lambda-specific errors
    pub fn generate_error_conversions(&self) -> String {
        LAMBDA_ERROR_CONVERSIONS.to_string()
    }
}

// Implementation templates
const APIGW_RESPONSE_IMPL: &str = r#"impl From<HandlerOutput> for ApiGatewayProxyResponse {
    fn from(output: HandlerOutput) -> Self {
        ApiGatewayProxyResponse {
            status_code: output.status_code,
            body: Some(serde_json::to_string(&output.body).unwrap()),
            headers: output.headers.into_iter().collect(),
            multi_value_headers: Default::default(),
            is_base64_encoded: false,
        }
    }
}"#;

const APIGW_V2_RESPONSE_IMPL: &str = r#"impl From<HandlerOutput> for ApiGatewayV2httpResponse {
    fn from(output: HandlerOutput) -> Self {
        ApiGatewayV2httpResponse {
            status_code: output.status_code,
            body: Some(serde_json::to_string(&output.body).unwrap()),
            headers: output.headers.into_iter().collect(),
            is_base64_encoded: Some(false),
            cookies: vec![],
        }
    }
}"#;

const APIGW_RESPONSE_BUILDER: &str = r#"pub struct ResponseBuilder {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status_code = status;
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn json<T: serde::Serialize>(mut self, data: &T) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_string(data)?);
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn build(self) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: self.status_code,
            headers: self.headers,
            body: self.body,
            multi_value_headers: Default::default(),
            is_base64_encoded: false,
        }
    }
}"#;

const APIGW_V2_RESPONSE_BUILDER: &str = r#"pub struct ResponseBuilderV2 {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl ResponseBuilderV2 {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status_code = status;
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn json<T: serde::Serialize>(mut self, data: &T) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_string(data)?);
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn build(self) -> ApiGatewayV2httpResponse {
        ApiGatewayV2httpResponse {
            status_code: self.status_code,
            headers: self.headers,
            body: self.body,
            is_base64_encoded: Some(false),
            cookies: vec![],
        }
    }
}"#;

const SQS_RESPONSE_BUILDER: &str = r#"pub struct SqsResponseBuilder {
    batch_item_failures: Vec<SqsBatchItemFailure>,
}

impl SqsResponseBuilder {
    pub fn new() -> Self {
        Self {
            batch_item_failures: Vec::new(),
        }
    }

    pub fn add_failure(mut self, item_identifier: String) -> Self {
        self.batch_item_failures.push(SqsBatchItemFailure {
            item_identifier,
        });
        self
    }

    pub fn build(self) -> SqsBatchResponse {
        SqsBatchResponse {
            batch_item_failures: self.batch_item_failures,
        }
    }
}"#;

const LAMBDA_ERROR_CONVERSIONS: &str = r#"#[derive(Debug, thiserror::Error)]
pub enum LambdaError {
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Handler error: {0}")]
    Handler(String),
    
    #[error("Runtime error: {0}")]
    Runtime(#[from] lambda_runtime::Error),
    
    #[error("HTTP error: {0}")]
    Http(String),
    
    #[error("Missing parameter: {0}")]
    MissingParameter(String),
}

// Note: lambda_runtime::Error conversion would be available when using the actual lambda_runtime crate
// impl From<LambdaError> for lambda_runtime::Error {
//     fn from(err: LambdaError) -> Self {
//         lambda_runtime::Error::from(err.to_string())
//     }
// }

// Automatic error chain generation for common Python patterns
impl From<&str> for LambdaError {
    fn from(msg: &str) -> Self {
        if msg.contains("KeyError") {
            LambdaError::MissingParameter(msg.to_string())
        } else if msg.contains("ValueError") {
            LambdaError::Handler(msg.to_string())
        } else {
            LambdaError::Handler(msg.to_string())
        }
    }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_mapping_retrieval() {
        let mapper = LambdaTypeMapper::new();
        
        let s3_mapping = mapper.get_event_mapping(&LambdaEventType::S3Event).unwrap();
        assert_eq!(s3_mapping.rust_type, "S3Event");
        assert_eq!(s3_mapping.aws_events_module, "s3");
    }

    #[test]
    fn test_response_mapping_retrieval() {
        let mapper = LambdaTypeMapper::new();
        
        let apigw_response = mapper.get_response_mapping(&LambdaEventType::ApiGatewayProxyRequest).unwrap();
        assert_eq!(apigw_response.rust_type, "ApiGatewayProxyResponse");
        assert!(apigw_response.conversion_impl.is_some());
    }

    #[test]
    fn test_eventbridge_custom_type() {
        let mapper = LambdaTypeMapper::new();
        
        let custom_event = LambdaEventType::EventBridgeEvent(Some("OrderEvent".to_string()));
        let mapping = mapper.get_event_mapping(&custom_event).unwrap();
        
        assert_eq!(mapping.rust_type, "EventBridgeEvent<serde_json::Value>");
    }

    #[test]
    fn test_type_conversion_rules() {
        let mapper = LambdaTypeMapper::new();
        let rules = mapper.get_type_conversion_rules();
        
        assert!(!rules.is_empty());
        
        let dict_rule = rules.iter().find(|r| r.python_pattern == "dict").unwrap();
        assert_eq!(dict_rule.rust_type, "T: Deserialize");
        assert!(dict_rule.serde_attribute.is_some());
    }

    #[test]
    fn test_custom_eventbridge_types_generation() {
        let mapper = LambdaTypeMapper::new();
        let generated = mapper.generate_custom_eventbridge_types("OrderEvent").unwrap();
        
        assert!(generated.contains("struct OrderEvent"));
        assert!(generated.contains("enum EventType"));
    }

    #[test]
    fn test_required_imports() {
        let mapper = LambdaTypeMapper::new();
        let imports = mapper.get_required_imports(&LambdaEventType::ApiGatewayProxyRequest);
        
        assert!(imports.iter().any(|i| i.contains("ApiGatewayProxyRequest")));
        assert!(imports.iter().any(|i| i.contains("HashMap")));
    }

    #[test]
    fn test_response_builders() {
        let mapper = LambdaTypeMapper::new();
        let builder = mapper.generate_response_builders(&LambdaEventType::ApiGatewayProxyRequest).unwrap();
        
        assert!(builder.contains("ResponseBuilder"));
        assert!(builder.contains("fn status"));
        assert!(builder.contains("fn json"));
    }

    #[test]
    fn test_error_conversions() {
        let mapper = LambdaTypeMapper::new();
        let errors = mapper.generate_error_conversions();
        
        assert!(errors.contains("enum LambdaError"));
        assert!(errors.contains("MissingParameter"));
        assert!(errors.contains("thiserror::Error"));
    }

    #[test]
    fn test_custom_mapping_addition() {
        let mut mapper = LambdaTypeMapper::new();
        
        let custom_event = LambdaEventType::Custom("MyEvent".to_string());
        let custom_mapping = EventTypeMapping {
            rust_type: "MyCustomEvent".to_string(),
            aws_events_module: "custom".to_string(),
            imports: vec!["use my_crate::MyCustomEvent;".to_string()],
            serde_attributes: vec![],
        };
        
        mapper.add_custom_event_mapping(custom_event.clone(), custom_mapping);
        
        let retrieved = mapper.get_event_mapping(&custom_event).unwrap();
        assert_eq!(retrieved.rust_type, "MyCustomEvent");
    }
}