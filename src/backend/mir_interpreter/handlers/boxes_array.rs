use super::*;
use crate::box_trait::NyashBox;

pub(super) fn try_handle_array_box(
    this: &mut MirInterpreter,
    dst: Option<ValueId>,
    box_val: ValueId,
    method: &str,
    args: &[ValueId],
) -> Result<bool, VMError> {
        let recv = this.reg_load(box_val)?;
        let recv_box_any: Box<dyn NyashBox> = match recv.clone() {
            VMValue::BoxRef(b) => b.share_box(),
            other => other.to_nyash_box(),
        };
        if let Some(ab) = recv_box_any
            .as_any()
            .downcast_ref::<crate::boxes::array::ArrayBox>()
        {
            match method {
                "birth" => {
                    // No-op constructor init
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "push" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("push expects 1 arg".into())); }
                    let val = this.reg_load(args[0])?.to_nyash_box();
                    let _ = ab.push(val);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "len" | "length" | "size" => {
                    let ret = ab.length();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "get" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("get expects 1 arg".into())); }
                    let idx = this.reg_load(args[0])?.to_nyash_box();
                    let ret = ab.get(idx);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "set" => {
                    if args.len() != 2 { return Err(VMError::InvalidInstruction("set expects 2 args".into())); }
                    let idx = this.reg_load(args[0])?.to_nyash_box();
                    let val = this.reg_load(args[1])?.to_nyash_box();
                    let _ = ab.set(idx, val);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
}
