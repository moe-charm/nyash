#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;
    use crate::mir::MirInstruction;

    // Regression for predecessor mis-selection on nested if inside loop in PHI-off mode
    // Path: cond == true && cond2 == false should observe inner else assignment at merge.
    #[test]
    fn nested_if_inside_loop_edges_copy_from_exiting_blocks() {
        // Force PHI-off
        std::env::set_var("NYASH_MIR_NO_PHI", "1");

        let code = r#"
            x = 0
            i = 0
            loop (i < 1) {
                i = i + 1
                if (1 == 1) {
                    if (1 == 0) {
                        x = 1
                    } else {
                        x = 2
                    }
                }
            }
            return x
        "#;

        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");

        // Find return block/value id
        let f = result.module.functions.get("main").expect("main");
        let (ret_block, out_v) = f
            .blocks
            .iter()
            .find_map(|(bid, bb)| match &bb.terminator {
                Some(MirInstruction::Return { value: Some(v) }) => Some((*bid, *v)),
                _ => None,
            })
            .expect("ret block");

        // Every predecessor must carry a Copy to the merged value in PHI-off mode
        let preds: Vec<_> = f.blocks.get(&ret_block).unwrap().predecessors.iter().copied().collect();
        assert!(!preds.is_empty(), "ret must have predecessors");
        for p in preds {
            let bb = f.blocks.get(&p).unwrap();
            let has_copy = bb
                .instructions
                .iter()
                .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
            assert!(has_copy, "missing Copy to merged value in predecessor {:?}", p);
        }
        // ret block must not contain Copy to out_v
        let merge_has_copy = f
            .blocks
            .get(&ret_block)
            .unwrap()
            .instructions
            .iter()
            .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
        assert!(!merge_has_copy, "merge/ret must not contain Copy to merged value");
    }
}
