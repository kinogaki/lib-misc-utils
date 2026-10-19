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

/// For every e: with r = floor(e log2 10) - 125 and g = G[e + 292],
/// (g - 1) × 2^r <= 10^e < g × 2^r, scaled to integers.
#[test]
fn g_is_the_rounded_up_scaled_power_of_ten() {
    for (i, &(high, low)) in tables::G.iter().enumerate() {
        let e = tables::G_MIN + i as i32;
        let r = ((e as i64 * 913_124_641_741) >> 38) as i32 - 125;
        let g = Big::from((high as u128) << 63 | low as u128);
        let g_minus_1 = Big::from(((high as u128) << 63 | low as u128) - 1);
        // Both sides times 2^max(-r,0) × 10^max(-e,0).
        let left_scale = pow(2, r.max(0) as u32).mul(&pow(10, (-e).max(0) as u32));
        let right = pow(10, e.max(0) as u32).mul(&pow(2, (-r).max(0) as u32));
        assert_ne!(
            g_minus_1.mul(&left_scale).cmp(&right),
            std::cmp::Ordering::Greater,
            "G[{i}] too large (e = {e})"
        );
        assert_eq!(
            right.cmp(&g.mul(&left_scale)),
            std::cmp::Ordering::Less,
            "G[{i}] too small (e = {e})"
        );
        assert!(
            g.bits() == 126 || g == Big::pow2(126),
            "G[{i}] has {} bits",
            g.bits()
        );
    }
}
