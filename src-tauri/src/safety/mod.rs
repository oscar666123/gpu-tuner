pub fn validate_power_limit(watts: f64, min: f64, max: f64) -> Result<(), String> {
    if !watts.is_finite() {
        return Err("Power limit must be a finite number.".to_string());
    }
    if watts < min || watts > max {
        return Err(format!(
            "Power limit {watts:.1} W is outside {min:.1}..{max:.1} W"
        ));
    }
    Ok(())
}

pub fn validate_clock_offset(value: i32, min: i32, max: i32, label: &str) -> Result<(), String> {
    if value < min || value > max {
        return Err(format!(
            "{label} offset {value} MHz is outside {min}..{max} MHz"
        ));
    }
    Ok(())
}
