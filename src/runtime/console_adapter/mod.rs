//! Console Adapter — centralized print normalization

use crate::backend::vm_types::VMValue;

pub fn print_value(v: &VMValue) {
    match v {
        VMValue::Void => { println!("null"); }
        VMValue::String(s) => { println!("{}", s); }
        VMValue::BoxRef(bx) => {
            if bx.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                println!("null");
                return;
            }
            if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                println!("{}", sb.value);
                return;
            }
            println!("{}", VMValue::BoxRef(bx.clone()).to_string());
        }
        _ => { println!("{}", v.to_string()); }
    }
}
