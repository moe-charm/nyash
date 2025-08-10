//! Tests for NyashBox trait implementations
use crate::box_trait::{NyashBox, StringBox, IntegerBox};
use crate::boxes::{ArrayBox, BufferBox, JSONBox, NyashFutureBox, NyashStreamBox, NyashResultBox};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_box_nyash_trait() {
        let mut array = ArrayBox::new();
        let str_box = Box::new(StringBox::new("test")) as Box<dyn NyashBox>;
        let int_box = Box::new(IntegerBox::new(42)) as Box<dyn NyashBox>;
        
        array.push(str_box);
        array.push(int_box);
        
        assert_eq!(array.type_name(), "ArrayBox");
        assert_eq!(array.len(), 2);
        
        let string_repr = array.to_string_box();
        assert!(string_repr.value.contains("test"));
        assert!(string_repr.value.contains("42"));
        
        // Test cloning
        let cloned = array.clone_box();
        assert_eq!(cloned.type_name(), "ArrayBox");
    }

    #[test]
    fn test_buffer_box_nyash_trait() {
        let buffer = BufferBox::from_vec(vec![1, 2, 3, 4, 5]);
        
        assert_eq!(buffer.type_name(), "BufferBox");
        assert_eq!(buffer.len(), 5);
        
        let string_repr = buffer.to_string_box();
        assert!(string_repr.value.contains("BufferBox(5 bytes)"));
        
        // Test cloning
        let cloned = buffer.clone_box();
        assert_eq!(cloned.type_name(), "BufferBox");
        
        // Test equality
        let other_buffer = BufferBox::from_vec(vec![1, 2, 3, 4, 5]);
        assert!(buffer.equals(&other_buffer).value);
    }

    #[test]
    fn test_json_box_nyash_trait() {
        let json_str = r#"{"name": "test", "value": 42}"#;
        let json_box = JSONBox::from_str(json_str).expect("Valid JSON");
        
        assert_eq!(json_box.type_name(), "JSONBox");
        
        let string_repr = json_box.to_string_box();
        assert!(string_repr.value.contains("test"));
        assert!(string_repr.value.contains("42"));
        
        // Test cloning
        let cloned = json_box.clone_box();
        assert_eq!(cloned.type_name(), "JSONBox");
        
        // Test equality
        let other_json = JSONBox::from_str(json_str).expect("Valid JSON");
        assert!(json_box.equals(&other_json).value);
    }

    #[test] 
    fn test_future_box_nyash_trait() {
        let future = NyashFutureBox::new();
        
        assert_eq!(future.type_name(), "NyashFutureBox");
        assert!(!future.ready());
        
        let string_repr = future.to_string_box();
        assert!(string_repr.value.contains("Future(pending)"));
        
        // Test setting result
        let result_box = Box::new(StringBox::new("completed")) as Box<dyn NyashBox>;
        future.set_result(result_box);
        
        assert!(future.ready());
        let result = future.get();
        assert_eq!(result.to_string_box().value, "completed");
    }

    #[test]
    fn test_stream_box_nyash_trait() {
        let mut stream = NyashStreamBox::from_data(vec![72, 101, 108, 108, 111]); // "Hello"
        
        assert_eq!(stream.type_name(), "NyashStreamBox");
        assert_eq!(stream.len(), 5);
        
        let string_repr = stream.to_string_box();
        assert!(string_repr.value.contains("NyashStreamBox(5 bytes"));
        
        // Test reading
        let mut buffer = [0u8; 3];
        let bytes_read = stream.read(&mut buffer).expect("Read should succeed");
        assert_eq!(bytes_read, 3);
        assert_eq!(&buffer, &[72, 101, 108]); // "Hel"
        
        // Test writing
        stream.write(&[33, 33]).expect("Write should succeed"); // "!!"
        assert_eq!(stream.len(), 7);
    }

    #[test]
    fn test_result_box_nyash_trait() {
        let success_result = NyashResultBox::new_ok(Box::new(StringBox::new("success")));
        
        assert_eq!(success_result.type_name(), "NyashResultBox");
        assert!(success_result.is_ok());
        assert!(!success_result.is_err());
        
        let string_repr = success_result.to_string_box();
        assert!(string_repr.value.contains("Ok(success)"));
        
        // Test error case
        let error_result = NyashResultBox::new_err(Box::new(StringBox::new("error")));
        assert!(!error_result.is_ok());
        assert!(error_result.is_err());
        
        let error_string = error_result.to_string_box();
        assert!(error_string.value.contains("Err(error)"));
    }

    #[test]
    fn test_box_id_uniqueness() {
        let box1 = ArrayBox::new();
        let box2 = ArrayBox::new();
        
        // Different instances should have different IDs
        assert_ne!(box1.box_id(), box2.box_id());
        
        // Same instance should have same ID
        let cloned = box1.clone_box();
        // Note: Clone creates new instance so ID will be different
        // but that's fine for our use case
        assert_eq!(cloned.type_name(), box1.type_name());
    }
}