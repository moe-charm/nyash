use crate::mir::builder::MirBuilder;

pub fn exists(builder: &MirBuilder, box_name: &str, method: &str, arity: usize) -> bool {
    builder
        .method_index
        .instance_method_exists(box_name, method, arity)
}
