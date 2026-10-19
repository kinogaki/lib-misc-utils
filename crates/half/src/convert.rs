//! Bit-level conversions between binary16 and binary32/binary64.

/// binary32 to binary16, round to nearest, ties to even.
pub const fn f32_to_f16(value: f32) -> u16 {
    let x = value.to_bits();
    let sign = ((x >> 16) & 0x8000) as u16;
    let exponent = ((x >> 23) & 0xff) as i32;
    let mantissa = x & 0x7f_ffff;
    if exponent == 0xff {
        if mantissa == 0 {
            return sign | 0x7c00;
        }
        // NaN: the payload's top 10 bits, quiet bit set.
        return sign | 0x7e00 | (mantissa >> 13) as u16;
    }
    let half_exponent = exponent - 127 + 15;
    if half_exponent >= 0x1f {
        return sign | 0x7c00;
    }
    if half_exponent <= 0 {
        // Subnormal (or zero): the value in units of 2^-24 is
        // (1.mantissa × 2^23) >> (126 - exponent).
        let shift = 126 - exponent;
        if shift > 24 {
            return sign;
        }
        let full = mantissa | 0x80_0000;
        return sign | round_shift(full as u64, shift as u32) as u16;
    }
    let rounded = round_shift(((half_exponent as u64) << 23) | mantissa as u64, 13);
    // A carry out of the mantissa moves to the next exponent (up to infinity).
    sign | rounded as u16
}

/// binary64 to binary16, round to nearest, ties to even (one rounding).
pub const fn f64_to_f16(value: f64) -> u16 {
    let x = value.to_bits();
    let sign = ((x >> 48) & 0x8000) as u16;
    let exponent = ((x >> 52) & 0x7ff) as i32;
    let mantissa = x & 0xf_ffff_ffff_ffff;
    if exponent == 0x7ff {
        if mantissa == 0 {
            return sign | 0x7c00;
        }
        return sign | 0x7e00 | (mantissa >> 42) as u16;
    }
    let half_exponent = exponent - 1023 + 15;
    if half_exponent >= 0x1f {
        return sign | 0x7c00;
    }
    if half_exponent <= 0 {
        let shift = 1022 + 52 - 23 - exponent;
        if shift > 53 {
            return sign;
        }
        let full = mantissa | (1 << 52);
        return sign | round_shift(full, shift as u32) as u16;
    }
    sign | round_shift(((half_exponent as u64) << 52) | mantissa, 42) as u16
}

/// `value >> shift`, rounded to nearest with ties to even.
const fn round_shift(value: u64, shift: u32) -> u64 {
    let quotient = value >> shift;
    let remainder = value & ((1 << shift) - 1);
    let halfway = 1 << (shift - 1);
    if remainder > halfway || (remainder == halfway && quotient & 1 == 1) {
        quotient + 1
    } else {
        quotient
    }
}

/// binary16 to binary32 (exact).
pub const fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exponent = ((bits >> 10) & 0x1f) as u32;
    let mantissa = (bits & 0x3ff) as u32;
    let magnitude = if exponent == 0x1f {
        if mantissa == 0 {
            0x7f80_0000
        } else {
            0x7fc0_0000 | (mantissa << 13)
        }
    } else if exponent == 0 {
        if mantissa == 0 {
            0
        } else {
            // Subnormal: mantissa × 2^-24, normal in binary32.
            let leading = mantissa.leading_zeros() - 21; // zeros above bit 10
            let shifted = (mantissa << leading) & 0x3ff;
            ((127 - 15 + 1 - leading) << 23) | (shifted << 13)
        }
    } else {
        ((exponent + 127 - 15) << 23) | (mantissa << 13)
    };
    f32::from_bits(sign | magnitude)
}

/// binary16 to binary64 (exact).
pub const fn f16_to_f64(bits: u16) -> f64 {
    let sign = ((bits & 0x8000) as u64) << 48;
    let exponent = ((bits >> 10) & 0x1f) as u64;
    let mantissa = (bits & 0x3ff) as u64;
    let magnitude = if exponent == 0x1f {
        if mantissa == 0 {
            0x7ff0_0000_0000_0000
        } else {
            0x7ff8_0000_0000_0000 | (mantissa << 42)
        }
    } else if exponent == 0 {
        if mantissa == 0 {
            0
        } else {
            let leading = (mantissa.leading_zeros() - 53) as u64; // zeros above bit 10
            let shifted = (mantissa << leading) & 0x3ff;
            ((1023 - 15 + 1 - leading) << 52) | (shifted << 42)
        }
    } else {
        ((exponent + 1023 - 15) << 52) | (mantissa << 42)
    };
    f64::from_bits(sign | magnitude)
}
