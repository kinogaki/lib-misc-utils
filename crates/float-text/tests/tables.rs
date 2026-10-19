//! The power-of-ten table (src/tables.rs) against its definition, with exact
//! big-integer arithmetic.

#[path = "../src/tables.rs"]
mod tables;

/// Little-endian 64-bit words.
#[derive(Clone, PartialEq, Debug)]
struct Big(Vec<u64>);

impl Big {
    fn from(v: u128) -> Big {
        Big::trim(vec![v as u64, (v >> 64) as u64])
    }
    fn trim(mut w: Vec<u64>) -> Big {
        while w.len() > 1 && *w.last().unwrap() == 0 {
            w.pop();
        }
        Big(w)
    }
    fn mul_small(&self, k: u64) -> Big {
        let mut carry = 0u128;
        let mut out = Vec::with_capacity(self.0.len() + 1);
        for &w in &self.0 {
            let p = w as u128 * k as u128 + carry;
            out.push(p as u64);
            carry = p >> 64;
        }
        out.push(carry as u64);
        Big::trim(out)
    }
    fn mul(&self, other: &Big) -> Big {
        let mut out = vec![0u64; self.0.len() + other.0.len() + 1];
        for (i, &a) in self.0.iter().enumerate() {
            let mut carry = 0u128;
            for (j, &b) in other.0.iter().enumerate() {
                let p = a as u128 * b as u128 + out[i + j] as u128 + carry;
                out[i + j] = p as u64;
                carry = p >> 64;
            }
            out[i + other.0.len()] = carry as u64;
        }
        Big::trim(out)
    }
    fn bits(&self) -> u32 {
        (self.0.len() as u32 - 1) * 64 + (64 - self.0.last().unwrap().leading_zeros())
    }
    fn pow2(n: u32) -> Big {
        let mut w = vec![0u64; n as usize / 64 + 1];
        w[n as usize / 64] = 1 << (n % 64);
        Big(w)
    }
    fn cmp(&self, other: &Big) -> std::cmp::Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

fn pow(base: u64, n: u32) -> Big {
    (0..n).fold(Big::from(1), |acc, _| acc.mul_small(base))
}

/// For every e: POW10[e + 293] = floor(10^e × 2^(127 - b)) with
/// b = floor(log2 10^e), i.e. 2^127 <= t, and t × 2^b <= 10^e × 2^127 < (t + 1) × 2^b
/// (scaled to integers), and b is what the search computes.
#[test]
fn pow10_is_the_truncated_128_bit_significand() {
    for (i, &(high, low)) in tables::POW10.iter().enumerate() {
        let e = tables::POW10_MIN + i as i32;
        let b = (e * 217_707) >> 16; // the search's floor_log2_pow10
        let t = Big::from((high as u128) << 64 | low as u128);
        let t1 = Big::from(((high as u128) << 64 | low as u128) + 1);
        assert_eq!(t.bits(), 128, "POW10[{i}] is not normalised (e = {e})");
        // Compare t × 2^b with 10^e × 2^127, both sides moved to integers.
        let ten = pow(10, e.unsigned_abs());
        let (left, right_base) = if e >= 0 {
            (Big::from(1), ten)
        } else {
            (ten, Big::from(1))
        };
        let scale_left = |x: &Big| {
            if b >= 0 {
                x.mul(&left).mul(&Big::pow2(b as u32))
            } else {
                x.mul(&left)
            }
        };
        let right = if b >= 0 {
            right_base.mul(&Big::pow2(127))
        } else {
            right_base.mul(&Big::pow2((127 - b) as u32))
        };
        assert_ne!(
            scale_left(&t).cmp(&right),
            std::cmp::Ordering::Greater,
            "POW10[{i}] too large (e = {e})"
        );
        assert_eq!(
            right.cmp(&scale_left(&t1)),
            std::cmp::Ordering::Less,
            "POW10[{i}] too small (e = {e})"
        );
    }
}
