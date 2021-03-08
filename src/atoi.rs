/// Convert ASCII to number directly.
pub fn atoi_u8(s: &[u8]) -> Option<u8> {
    let mut number: u8 = 0;
    let mut i = 0;
    let mut base = 1;

    while i < s.len() {
        match s[i] {
            digit @ b'0'..=b'9' => {
                number = match number
                    .checked_mul(base)
                    .and_then(|n| u8::checked_add(n, digit - b'0'))
                {
                    Some(number) => number,
                    None => return None,
                };
                base = 10;
            }
            _ => {
                return None;
            }
        }

        i += 1;
    }

    Some(number)
}
