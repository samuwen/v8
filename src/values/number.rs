use core::f64;
use std::ops::Neg;

use crate::{Interpreter, values::JSValue};

fn is_even(x: f64) -> bool {
    x % 2.0 == 0.0
}

fn is_odd_and_integral(x: f64) -> bool {
    let integral = is_integral(x);
    let is_odd = !is_even(x);
    is_odd && integral
}

fn is_integral(x: f64) -> bool {
    let fract = x.fract();
    return fract == 0.0;
}

fn to_integral(x: f64) -> i32 {
    let truncated = x.trunc();
    truncated as i32
}

pub fn unary_minus(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    x.neg()
}

pub fn bitwise_not(x: f64, interpreter: &mut Interpreter) -> i32 {
    let value = JSValue::new_number(&x);
    let int = value
        .to_int_32(interpreter)
        .expect("Invalid number returned from to_int_32");
    !int
}

pub fn exponentiate(base: f64, exponent: f64) -> f64 {
    // 1
    if exponent.is_nan() {
        return f64::NAN;
    }
    // 2
    if exponent == 0.0 || exponent == -0.0 {
        return 1.0;
    }
    // 3
    if base.is_nan() {
        return f64::NAN;
    }
    // 4
    if base == f64::INFINITY {
        if exponent > 0.0 {
            return f64::INFINITY;
        }
        return 0.0;
    }
    // 5
    if base == f64::NEG_INFINITY {
        if exponent > 0.0 {
            if is_odd_and_integral(exponent) {
                return f64::NEG_INFINITY;
            }
            return f64::INFINITY;
        }
        if is_odd_and_integral(exponent) {
            return -0.0;
        }
        return 0.0;
    }

    // 6
    if base == 0.0 {
        if exponent > 0.0 {
            return 0.0;
        }
        return f64::INFINITY;
    }

    // 7
    if base == -0.0 {
        if exponent > 0.0 {
            if is_odd_and_integral(exponent) {
                return -0.0;
            }
            return 0.0;
        }
        if is_odd_and_integral(exponent) {
            return f64::NEG_INFINITY;
        }
        return f64::INFINITY;
    }

    // 9
    if exponent == f64::INFINITY {
        let abs_base = base.abs();
        if abs_base > 1.0 {
            return f64::INFINITY;
        }
        if abs_base == 1.0 {
            return f64::NAN;
        }
        if abs_base < 1.0 {
            return 0.0;
        }
    }

    // 10
    if exponent == f64::NEG_INFINITY {
        let abs_base = base.abs();
        if abs_base > 1.0 {
            return 0.0;
        }
        if abs_base == 1.0 {
            return f64::NAN;
        }
        if abs_base < 1.0 {
            return f64::INFINITY;
        }
    }

    // 12
    if base < -0.0 && !is_integral(exponent) {
        return f64::NAN;
    }

    base.powf(exponent)
}

pub fn multiply(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }

    if x.is_infinite() {
        if y == 0.0 || y == -0.0 {
            return f64::NAN;
        }
        if y > 0.0 {
            return x;
        }
        return -x;
    }

    if y.is_infinite() {
        if x == 0.0 || x == -0.0 {
            return f64::NAN;
        }
        if x > 0.0 {
            return y;
        }
        return -y;
    }

    if x == -0.0 {
        if y <= -0.0 {
            return 0.0;
        }
        return -0.0;
    }

    if y == -0.0 {
        if x < -0.0 {
            return 0.0;
        }
        return -0.0;
    }

    x * y
}

pub fn divide(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }

    if x.is_infinite() {
        if y.is_infinite() {
            return f64::NAN;
        }
        if y >= 0.0 {
            return x;
        }
        return -x;
    }

    if y == f64::INFINITY {
        if x >= 0.0 {
            return 0.0;
        }
        return -0.0;
    }

    if y == f64::NEG_INFINITY {
        if x > 0.0 {
            return -0.0;
        }
        return 0.0;
    }

    if x == 0.0 || x == -0.0 {
        if y == 0.0 || y == -0.0 {
            return f64::NAN;
        }
        if y > 0.0 {
            return x;
        }
        return -x;
    }

    if y == 0.0 {
        if x > 0.0 {
            return f64::INFINITY;
        }
        return f64::NEG_INFINITY;
    }

    if y == -0.0 {
        if x > 0.0 {
            return f64::NEG_INFINITY;
        }
        return f64::INFINITY;
    }

    x / y
}

pub fn remainder(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }

    if x.is_infinite() {
        return f64::NAN;
    }

    if y.is_infinite() {
        return x;
    }

    if y == 0.0 || y == -0.0 {
        return f64::NAN;
    }

    if x == 0.0 || y == 0.0 {
        return x;
    }

    let quotient = x / y;
    let q = quotient.trunc();
    let r = x - (y * q);
    if r == 0.0 && x < -0.0 {
        return -0.0;
    }

    return r;
}

pub fn add(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if x == f64::INFINITY && y == f64::NEG_INFINITY {
        return f64::NAN;
    }
    if y == f64::INFINITY && x == f64::NEG_INFINITY {
        return f64::NAN;
    }
    if x.is_infinite() {
        return x;
    }
    if y.is_infinite() {
        return y;
    }

    x + y
}

pub fn subtract(x: f64, y: f64) -> f64 {
    add(x, unary_minus(y))
}

pub fn left_shift(x: f64, y: f64, interpreter: &mut Interpreter) -> i32 {
    let x_value = JSValue::new_number(&x);
    let y_value = JSValue::new_number(&y);
    let l_num = x_value
        .to_int_32(interpreter)
        .expect("Invalid number returned from to_int_32");
    let r_num = y_value
        .to_uint_32(interpreter)
        .expect("Invalid number returned from to_int_32");

    let shift_count = r_num % 32;
    l_num << shift_count as i32
}

pub fn signed_right_shift(x: f64, y: f64, interpreter: &mut Interpreter) -> i32 {
    let x_value = JSValue::new_number(&x);
    let y_value = JSValue::new_number(&y);
    let l_num = x_value
        .to_int_32(interpreter)
        .expect("Invalid number returned from to_int_32");
    let r_num = y_value
        .to_uint_32(interpreter)
        .expect("Invalid number returned from to_int_32");
    let shift_count = r_num as i32 % 32;
    l_num >> shift_count
}

pub fn unsigned_right_shift(x: f64, y: f64, interpreter: &mut Interpreter) -> u32 {
    let x_value = JSValue::new_number(&x);
    let y_value = JSValue::new_number(&y);
    let l_num = x_value
        .to_int_32(interpreter)
        .expect("Invalid number returned from to_int_32") as u32;
    let r_num = y_value
        .to_uint_32(interpreter)
        .expect("Invalid number returned from to_int_32");
    let shift_count = r_num % 32;
    l_num >> shift_count
}

pub fn less_than(x: f64, y: f64) -> bool {
    if x.is_nan() || y.is_nan() {
        return false;
    }
    if x == y {
        return false;
    }
    if x == 0.0 && y == -0.0 {
        return false;
    }
    if x == -0.0 && y == 0.0 {
        return false;
    }
    if x == f64::INFINITY {
        return false;
    }
    if y == f64::INFINITY {
        return true;
    }
    if y == f64::NEG_INFINITY {
        return false;
    }
    if x == f64::NEG_INFINITY {
        return true;
    }
    return x < y;
}

pub fn equal(x: f64, y: f64) -> bool {
    if x.is_nan() || y.is_nan() {
        return false;
    }
    if x == y {
        return true;
    }
    if x == 0.0 && y == -0.0 {
        return true;
    }
    if x == -0.0 && y == 0.0 {
        return true;
    }

    false
}

pub fn same_value(x: f64, y: f64) -> bool {
    if x.is_nan() && y.is_nan() {
        return true;
    }
    if x == 0.0 && y == -0.0 {
        return false;
    }
    if x == -0.0 && y == 0.0 {
        return false;
    }
    if x == y {
        return true;
    }

    false
}

pub fn same_value_zero(x: f64, y: f64) -> bool {
    if x.is_nan() && y.is_nan() {
        return true;
    }
    if x == 0.0 && y == -0.0 {
        return true;
    }
    if x == -0.0 && y == 0.0 {
        return true;
    }
    if x == y {
        return true;
    }

    false
}

#[derive(Debug)]
pub enum BitwiseOp {
    And,
    Xor,
    Or,
}
fn bitwise_op(op: BitwiseOp, x: f64, y: f64, interpreter: &mut Interpreter) -> i32 {
    let x_value = JSValue::new_number(&x);
    let y_value = JSValue::new_number(&y);
    let l_num = x_value
        .to_int_32(interpreter)
        .expect("Invalid number returned from to_int_32");
    let r_num = y_value
        .to_int_32(interpreter)
        .expect("Invalid number returned from to_int_32");
    match op {
        BitwiseOp::And => l_num & r_num,
        BitwiseOp::Xor => l_num ^ r_num,
        BitwiseOp::Or => l_num | r_num,
    }
}

pub fn bitwise_and(x: f64, y: f64, interpreter: &mut Interpreter) -> i32 {
    return bitwise_op(BitwiseOp::And, x, y, interpreter);
}

pub fn bitwise_xor(x: f64, y: f64, interpreter: &mut Interpreter) -> i32 {
    return bitwise_op(BitwiseOp::Xor, x, y, interpreter);
}

pub fn bitwise_or(x: f64, y: f64, interpreter: &mut Interpreter) -> i32 {
    return bitwise_op(BitwiseOp::Or, x, y, interpreter);
}

pub fn to_string(x: f64, radix: u8) -> String {
    x.to_string()
}
