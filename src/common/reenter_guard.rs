/*!
 * reenter_guard – small helper to track re-entrancy by key
 */
use std::collections::HashMap;

pub fn bump_and_check(map: &mut HashMap<String, usize>, key: &str, limit: Option<usize>) -> Result<usize, String> {
    let c = map.entry(key.to_string()).or_insert(0);
    *c += 1;
    if let Some(lim) = limit { if *c > lim { return Err(format!("reentrancy guard: {} depth>{}", key, lim)); } }
    Ok(*c)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn depth_increments_and_limit() {
        let mut m = HashMap::new();
        assert_eq!(bump_and_check(&mut m, "k", Some(3)).unwrap(), 1);
        assert_eq!(bump_and_check(&mut m, "k", Some(3)).unwrap(), 2);
        assert_eq!(bump_and_check(&mut m, "k", Some(3)).unwrap(), 3);
        let e = bump_and_check(&mut m, "k", Some(3)).unwrap_err();
        assert!(e.contains("reentrancy guard"));
    }
}
