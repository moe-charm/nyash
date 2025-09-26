/// Parse debug fuel value ("unlimited" or numeric)
pub fn parse_debug_fuel(value: &str) -> Option<usize> {
    if value == "unlimited" {
        None
    } else {
        value.parse::<usize>().ok()
    }
}

