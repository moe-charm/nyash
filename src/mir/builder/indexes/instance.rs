use crate::mir::builder::MirBuilder;

/// InstanceMethodIndexBox — register and query instance methods (Box, method, arity)
pub fn register(builder: &mut MirBuilder, box_name: &str, method: &str, arity: usize) {
    builder
        .instance_method_index
        .insert((box_name.to_string(), method.to_string(), arity));
}

pub fn exists(builder: &MirBuilder, box_name: &str, method: &str, arity: usize) -> bool {
    builder
        .instance_method_index
        .contains(&(box_name.to_string(), method.to_string(), arity))
}

