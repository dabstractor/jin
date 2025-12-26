# Rust Unified Value Type Enum Patterns for Multiple Data Formats

## 1. serde_json::Value Enum Analysis

### Structure Overview
```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

### Key Characteristics:
- **Recursive structure**: Contains Vec<Value> and Map<String, Value>
- **Box-free**: Uses direct types (String, Vec, Map) which are size-known
- **Primitives**: Handles basic JSON types (null, bool, numbers, strings)
- **Collection types**: Arrays and objects as first-class citizens

### Memory Management:
- `Vec<Value>` and `Map<String, Value>` are heap-allocated but size-known at compile time
- No need for Box in this case because the collection types have fixed size
- Each Value enum variant is a simple enum with fixed-size discriminant

## 2. Unified Value Type Enum Design

### Basic Pattern for Multiple Formats
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),          // 64-bit integers should suffice for most use cases
    Float(f64),            // IEEE 754 double precision
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),

    // Format-specific extensions (optional)
    // Binary(Vec<u8>),      // For binary data in formats like YAML
    // Timestamp(DateTime), // For datetime handling
}

// Derive macros for serialization
#[derive(Serialize, Deserialize)]
#[serde(untagged)] // Allows seamless conversion between types
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Null,
}
```

### Advanced Pattern with Format Extensions
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    // Primitive types
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),

    // Collection types
    Array(ArrayValue),
    Object(ObjectValue),

    // Format-specific variants
    // For YAML: handles binary, timestamps, etc.
    Binary(Vec<u8>),
    Timestamp(chrono::DateTime<chrono::Utc>),

    // Specialized types
    Regex(regex::Regex),  // Regex patterns
    Glob(String),        // Glob patterns
}

// Type aliases for clarity
pub type ArrayValue = Vec<Value>;
pub type ObjectValue = Map<String, Value>;
```

## 3. Recursive Enum Box Patterns

### When to Use Box
For truly recursive structures where self-reference is unavoidable:

```rust
// Example: JSON-like structure with potential circular references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonNode {
    Primitive(PrimitiveValue),
    Object {
        name: String,
        children: Vec<JsonNode>,
        parent: Option<Box<JsonNode>>, // Box for self-reference
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}
```

### Box Allocation Patterns
```rust
// Pattern 1: Tree structure with Box children
#[derive(Debug, Clone, PartialEq)]
enum Tree {
    Leaf(String),
    Node {
        value: String,
        left: Box<Tree>,
        right: Box<Tree>,
    },
}

// Pattern 2: Graph with Rc/Arc for shared ownership
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
enum GraphNode {
    Terminal { id: usize, data: String },
    Internal {
        id: usize,
        children: Vec<Rc<GraphNode>>, // Rc allows shared references
    },
}
```

## 4. Derive Macros Required

### Essential Derives
```rust
#[derive(Debug, Clone, PartialEq)]
// Basic inspection and equality checking
```

### Serialization Derives
```rust
#[derive(Serialize, Deserialize)]
// Standard Serde support for JSON, YAML, TOML
```

### Additional Useful Derives
```rust
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    From,              // Conversion from primitive types
    Into,             // Conversion to other types
)]
```

### Custom Derive with serde_as
For complex scenarios:
```rust
use serde_as;

#[derive(Serialize, Deserialize)]
#[serde_as(as = "Vec<serde_json::Value>")]
pub struct UnifiedData {
    pub items: Vec<Value>,
}
```

## 5. Memory Management Patterns

### Strategy 1: Direct Storage (like serde_json)
```rust
// Pros:
// - No heap allocation overhead
// - Simple to implement
// - Good cache locality
//
// Cons:
// - Can lead to large stack usage for deep structures
// - Limited by stack size
```

### Strategy 2: Box for Indirection
```rust
// Pros:
// - Prevents stack overflow
// - Works with arbitrary recursion depth
//
// Cons:
// - Additional heap allocation overhead
// - Pointer chasing can impact cache performance
```

### Strategy 3: Rc/Arc for Shared Ownership
```rust
// Use Rc for single-threaded scenarios:
use std::rc::Rc;

#[derive(Debug, Clone)]
enum SharedValue {
    Primitive(Primitive),
    Shared(Rc<SharedValue>),
}

// Use Arc for multi-threaded scenarios:
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ThreadSafeValue {
    Primitive(Primitive),
    Shared(Arc<ThreadSafeValue>),
}
```

### Strategy 4: Arena Allocation
For performance-critical applications:
```rust
pub struct ValueArena {
    values: Vec<Value>,
    objects: Vec<ObjectNode>,
}

impl ValueArena {
    pub fn create_value(&mut self, value: Value) -> ValueRef {
        // Arena-based allocation for efficient memory usage
    }
}
```

## 6. Practical Implementation Examples

### Example 1: Basic Value Type
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

// Convert from primitives
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

// etc. for other types
```

### Example 2: Advanced with Merge Operations
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(ObjectMap),
}

impl Value {
    pub fn merge(&self, other: &Value) -> Value {
        match (self, other) {
            // Both primitives - prefer other
            (Value::Null, _) => other.clone(),
            (_, Value::Null) => self.clone(),

            // Both objects - merge recursively
            (Value::Object(map1), Value::Object(map2)) => {
                let mut merged = map1.clone();
                for (key, value2) in map2 {
                    if let Some(value1) = merged.get(key) {
                        merged.insert(key.clone(), value1.merge(value2));
                    } else {
                        merged.insert(key.clone(), value2.clone());
                    }
                }
                Value::Object(merged)
            }

            // Arrays - concatenate
            (Value::Array(arr1), Value::Array(arr2)) => {
                Value::Array([arr1.clone(), arr2.clone()].concat())
            }

            // Otherwise - prefer other
            (_, other) => other.clone(),
        }
    }
}
```

### Example 3: Integration with Multiple Formats
```rust
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

impl TryFrom<JsonValue> for Value {
    type Error = anyhow::Error;

    fn try_from(json: JsonValue) -> Result<Self, Self::Error> {
        match json {
            JsonValue::Null => Ok(Value::Null),
            JsonValue::Bool(b) => Ok(Value::Boolean(b)),
            JsonValue::Number(n) => {
                if n.is_i64() {
                    Ok(Value::Integer(n.as_i64().unwrap()))
                } else {
                    Ok(Value::Float(n.as_f64().unwrap()))
                }
            }
            JsonValue::String(s) => Ok(Value::String(s)),
            JsonValue::Array(arr) => {
                let converted: Result<Vec<Value>, _> = arr
                    .into_iter()
                    .map(Value::try_from)
                    .collect();
                Ok(Value::Array(converted?))
            }
            JsonValue::Object(obj) => {
                let converted: Result<ObjectMap, _> = obj
                    .into_iter()
                    .map(|(k, v)| Ok((k, Value::try_from(v)?)))
                    .collect();
                Ok(Value::Object(converted?))
            }
        }
    }
}
```

## 7. Best Practices and Patterns

### 1. Enum Design Principles
- **Keep it simple**: Start with minimal set of variants
- **Add incrementally**: Only add format-specific variants when needed
- **Be consistent**: Use similar patterns across all variants
- **Document decisions**: Clearly explain why certain types were chosen

### 2. Memory Considerations
- **Profile memory usage**: Use `std::mem::size_of` to check enum size
- **Consider trade-offs**: Balance between performance and flexibility
- **Use arena allocation**: For large, structured data
- **Lazy evaluation**: For expensive computations

### 3. Performance Optimization
- **Zero-cost abstractions**: Ensure your implementation doesn't add runtime overhead
- **Batch operations**: Process multiple values together when possible
- **Use const generics**: For fixed-size collections where applicable
- **Custom allocators**: For high-performance scenarios

### 4. Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum ValueError {
    #[error("Type mismatch: expected {expected}, found {actual}")]
    TypeMismatch { expected: &'static str, actual: &'static str },
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}
```

### 5. Testing Strategies
- **Property-based testing**: Use proptest for edge cases
- **Round-trip testing**: Serialize → deserialize → verify equivalence
- **Performance benchmarking**: Compare against native implementations
- **Cross-format compatibility**: Ensure values work across all supported formats

## 8. Recommended Additional Reading

1. **Rust Book Enums**: https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
2. **Serde Documentation**: https://serde.rs/
3. **serde_json Value implementation**: https://docs.rs/serde_json/latest/serde_json/enum.Value.html
4. **Rust Memory Management**: https://doc.rust-lang.org/book/ch15-00-smart-pointers.html
5. **Recursive types in Rust**: https://rust-lang.github.io/rfcs/0788-box-syntax-and-patterns.html

## 9. Future Extensions

The unified value type can be extended to support:
- **Binary data**: For formats that support it
- **Datetime handling**: With timezone support
- **UUID handling**: For identifier fields
- **Custom types**: Via extension traits
- **Schema validation**: Integrated with JSON Schema
- **JSON Path support**: For query operations
- **Transformation pipelines**: For complex data processing