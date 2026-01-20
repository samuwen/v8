use crate::{
    Interpreter,
    values::{JSResult, JSValue},
};

pub fn to_int_32(value: JSValue, interpreter: &mut Interpreter) -> JSResult<i32> {
    let number = value.to_number(interpreter)?;
    if number.is_infinite() || number == 0.0 || number == -0.0 {
        return Ok(0);
    }
    let int = number.floor() as i32;
    let rhs_mod = 2i32.pow(32);
    let int32bit = int % rhs_mod;
    if int32bit >= 2i32.pow(31) {
        return Ok(int32bit - rhs_mod);
    }

    Ok(int32bit)
}

/*
If number is not finite or number is either +0𝔽 or -0𝔽, return +0𝔽.
3. 3. Let int be truncate(ℝ(number)).
4. 4. Let int32bit be int modulo 2****32.
5. 5. If int32bit ≥ 2****31, return 𝔽(int32bit - 2****32); otherwise return 𝔽(int32bit).
 */
