// Basic operations
pub fn add(left: f64, right: f64) -> f64 {
    left + right
}

pub fn subtract(left: f64, right: f64) -> f64 {
    left - right
}

pub fn multiply(left: f64, right: f64) -> f64 {
    left * right
}

pub fn divide(left: f64, right: f64) -> f64 {
    if right == 0.0 || right == -0.0 {
        panic!("Cannot divide by 0");
    } else if right == f64::INFINITY || right == f64::NEG_INFINITY {
        panic!("Cannot divide by infinity");
    } else {
        left / right
    }
}

pub fn power(left: f64, right: f64) -> f64 {
    libm::pow(left, right)
}

// Comparison operations
pub fn equals(left: f64, right: f64) -> bool {
    (left - right).abs() < f64::EPSILON
}

pub fn not_equals(left: f64, right: f64) -> bool {
    !equals(left, right)
}

pub fn less_than(left: f64, right: f64) -> bool {
    left < right
}

pub fn greater_than(left: f64, right: f64) -> bool {
    left > right
}

pub fn less_than_equal(left: f64, right: f64) -> bool {
    left <= right
}

pub fn greater_than_equal(left: f64, right: f64) -> bool {
    left >= right
}
