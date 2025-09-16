use nyash_rust::box_trait::{BoolBox, IntegerBox, NyashBox, StringBox, VoidBox};
use nyash_rust::grammar::engine;

fn classify_value(b: &dyn NyashBox) -> &'static str {
    if nyash_rust::runtime::semantics::coerce_to_string(b).is_some() {
        "String"
    } else if nyash_rust::runtime::semantics::coerce_to_i64(b).is_some() {
        // coerce_to_i64 succeeds for integers and some numeric-like boxes
        // For this snapshot, we only feed IntegerBox so "Integer" is fine
        "Integer"
    } else if b.as_any().downcast_ref::<BoolBox>().is_some() {
        "Bool"
    } else {
        "Other"
    }
}

fn actual_add_result(left: &dyn NyashBox, right: &dyn NyashBox) -> &'static str {
    // Mirror current interpreter semantics succinctly:
    // 1) If either is string-like => String
    if nyash_rust::runtime::semantics::coerce_to_string(left).is_some()
        || nyash_rust::runtime::semantics::coerce_to_string(right).is_some()
    {
        return "String";
    }
    // 2) If both are i64-coercible => Integer
    if nyash_rust::runtime::semantics::coerce_to_i64(left).is_some()
        && nyash_rust::runtime::semantics::coerce_to_i64(right).is_some()
    {
        return "Integer";
    }
    // 3) Otherwise error（ここでは Error として表現）
    "Error"
}

#[test]
fn snapshot_add_rules_align_with_current_semantics() {
    let eng = engine::get();
    // Prepare sample operands for each class
    let s = StringBox::new("a".to_string());
    let i = IntegerBox::new(1);
    let b = BoolBox::new(true);
    let v = VoidBox::new();
    let vals: Vec<(&str, Box<dyn NyashBox>)> = vec![
        ("String", Box::new(s)),
        ("Integer", Box::new(i)),
        ("Bool", Box::new(b)),
        ("Other", Box::new(v)),
    ];

    for (li, l) in &vals {
        for (ri, r) in &vals {
            let lty = classify_value(l.as_ref());
            let rty = classify_value(r.as_ref());
            let actual = actual_add_result(l.as_ref(), r.as_ref());
            let expect = eng.decide_add_result(lty, rty).map(|(res, _)| res);
            if let Some(res) = expect {
                if actual == "Error" {
                    panic!(
                        "grammar provides rule for {}+{} but actual semantics error",
                        li, ri
                    );
                } else {
                    assert_eq!(
                        res, actual,
                        "grammar expect {} + {} => {}, but actual => {}",
                        li, ri, res, actual
                    );
                }
            } else {
                assert_eq!(
                    actual, "Error",
                    "grammar has no rule for {}+{}, but actual => {}",
                    li, ri, actual
                );
            }
        }
    }
}
