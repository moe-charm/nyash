use crate::mir::builder::MirBuilder;

pub fn exists(builder: &MirBuilder, box_name: &str, method: &str, arity: usize) -> bool {
    builder
        .instance_method_index
        .contains(&(box_name.to_string(), method.to_string(), arity))
}

