//! The GS1 Mod-10 check digit.
//!
//! One algorithm serves every GTIN length. GS1 defines the check digit over
//! the positions of the *canonical* form, and leading zeros contribute
//! nothing to the weighted sum, so computing it over a zero-padded GTIN-14
//! gives the same answer as computing it over the GTIN-8, GTIN-12 or GTIN-13
//! that padded to it. barkod therefore only ever computes it one way.

/// The check digit that *should* terminate a 13-digit body.
///
/// Weights alternate 3, 1 from the rightmost body digit leftwards, and the
/// check digit is whatever completes the weighted sum to the next multiple of
/// ten.
///
/// `body` must be 13 ASCII digits. It always is: the only caller holds a
/// [`Gtin14`](crate::Gtin14), which cannot exist otherwise.
pub(crate) fn check_digit(body: &[u8; 13]) -> u8 {
    let value = |b: &u8| u32::from(b - b'0');
    // The rightmost body digit is at index 12 and takes weight 3; weights
    // alternate leftwards, which lands weight 3 on every even index.
    let weighted: u32 = body.iter().step_by(2).map(value).sum::<u32>() * 3;
    let plain: u32 = body.iter().skip(1).step_by(2).map(value).sum();
    let sum = weighted + plain;
    b'0' + u8::try_from((10 - (sum % 10)) % 10).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::check_digit;

    fn body_of(s: &str) -> [u8; 13] {
        let mut out = [b'0'; 13];
        out.copy_from_slice(&s.as_bytes()[..13]);
        out
    }

    #[test]
    fn matches_known_good_gtins() {
        // Real production values whose check digit is correct. Each one was
        // verified against an independent implementation before being written
        // down here, rather than assumed correct because it looked real.
        for gtin in ["07350053850019", "00010700021526", "02112800000000"] {
            let expected = gtin.as_bytes()[13];
            assert_eq!(
                check_digit(&body_of(gtin)),
                expected,
                "should be valid: {gtin}"
            );
        }
    }

    #[test]
    fn known_bad_check_digits_stay_bad() {
        // Also real, also in production, and deliberately kept as fixtures:
        // `07390525907745` is the value the audit issues use throughout as
        // the "genuine" one, and its check digit does not validate. That is
        // the whole reason barkod classifies rather than rejects.
        for (gtin, expected) in [("07390525907745", b'2'), ("00000007350001", b'9')] {
            assert_eq!(check_digit(&body_of(gtin)), expected);
            assert_ne!(check_digit(&body_of(gtin)), gtin.as_bytes()[13]);
        }
    }

    #[test]
    fn all_zeros_checks_to_zero() {
        assert_eq!(check_digit(&body_of("0000000000000")), b'0');
    }
}
