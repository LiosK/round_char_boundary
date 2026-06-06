use core::hint::assert_unchecked;

const N: usize = 20;

#[unsafe(no_mangle)]
pub fn floor_runtime_std(s: &str, index: usize) -> usize {
    s.floor_char_boundary(index)
}

#[unsafe(no_mangle)]
pub fn floor_runtime_unrolled(s: &str, index: usize) -> usize {
    s.floor_char_boundary_unrolled(index)
}

#[unsafe(no_mangle)]
pub fn floor_runtime_mask(s: &str, index: usize) -> usize {
    s.floor_char_boundary_mask(index)
}

#[unsafe(no_mangle)]
pub fn floor_const_std(s: &str) -> usize {
    s.floor_char_boundary(N)
}

#[unsafe(no_mangle)]
pub fn floor_const_unrolled(s: &str) -> usize {
    s.floor_char_boundary_unrolled(N)
}

#[unsafe(no_mangle)]
pub fn floor_const_mask(s: &str) -> usize {
    s.floor_char_boundary_mask(N)
}

#[unsafe(no_mangle)]
pub fn ceil_runtime_std(s: &str, index: usize) -> usize {
    s.ceil_char_boundary(index)
}

#[unsafe(no_mangle)]
pub fn ceil_runtime_unrolled(s: &str, index: usize) -> usize {
    s.ceil_char_boundary_unrolled(index)
}

#[unsafe(no_mangle)]
pub fn ceil_const_std(s: &str) -> usize {
    s.ceil_char_boundary(N)
}

#[unsafe(no_mangle)]
pub fn ceil_const_unrolled(s: &str) -> usize {
    s.ceil_char_boundary_unrolled(N)
}

trait StrExt {
    fn floor_char_boundary_unrolled(&self, index: usize) -> usize;
    fn floor_char_boundary_mask(&self, index: usize) -> usize;
    fn ceil_char_boundary_unrolled(&self, index: usize) -> usize;
}

impl StrExt for str {
    #[inline]
    fn floor_char_boundary_unrolled(&self, index: usize) -> usize {
        if index >= self.len() {
            self.len()
        } else {
            // Unlike `ceil_char_boundary`, the loop is unrolled manually to prevent the compiler
            // from generating excessive unrolled loop bodies when `index` is statically known.
            let mut i = index;
            // The first byte of `&str` must always be a char boundary, so we can assume `i > 0`
            // below if `self.as_bytes()[i]` is not a char boundary.
            debug_assert!(self.as_bytes()[0].is_utf8_char_boundary());
            if !self.as_bytes()[i].is_utf8_char_boundary() {
                // SAFETY: `self.as_bytes()[0]` is always a char boundary with valid `&str`
                unsafe { assert_unchecked(i > 0) };
                i -= 1;
                if !self.as_bytes()[i].is_utf8_char_boundary() {
                    // SAFETY: `self.as_bytes()[0]` is always a char boundary with valid `&str`
                    unsafe { assert_unchecked(i > 0) };
                    i -= 1;
                    if !self.as_bytes()[i].is_utf8_char_boundary() {
                        debug_assert!(i > 0);
                        i -= 1;
                        // The character boundary will be within four bytes of the index
                        debug_assert!(self.as_bytes()[i].is_utf8_char_boundary());
                    }
                }
            }
            i
        }
    }

    #[inline]
    fn floor_char_boundary_mask(&self, index: usize) -> usize {
        if index >= self.len() {
            return self.len();
        }

        // A UTF-8 character is at most four bytes long, so the character boundary will reside
        // within `self.as_bytes()[index - 3..=index]` if `index < self.len()`.
        if index >= 3 {
            // Read the four bytes as `u32`, use bitwise operations to mark boundary bytes, and
            // count the number of leading/trailing zeros to find the last character boundary.

            // SAFETY: `index < self.len()` and `index >= 3` ensure that the four bytes starting at
            // `index - 3` are within bounds of `self`. `[u8; 4]` has the same alignment as `str`.
            let bytes = unsafe { self.as_ptr().add(index - 3).cast::<[u8; 4]>().read() };
            // Mask the top two bits of each byte and XOR with 0x80, leaving UTF-8 continuation
            // bytes (0b10xxxxxx) zero and everything else non-zero.
            let flags = (u32::from_ne_bytes(bytes) & 0xC0C0_C0C0) ^ 0x8080_8080;
            debug_assert!(flags != 0);
            #[cfg(target_endian = "little")]
            let offset = flags.leading_zeros() / 8;
            #[cfg(target_endian = "big")]
            let offset = flags.trailing_zeros() / 8;
            index - offset as usize
        } else {
            if self.as_bytes()[index].is_utf8_char_boundary() {
                index
            } else {
                // The first byte of `str` must always be a character boundary, so we can assume
                // `index > 0` here. Then, `index` is 2 or 1, and `self.as_bytes()[2]` is checked
                // not to be a character boundary, so the answer will be 1 or 0.
                debug_assert!(self.as_bytes()[0].is_utf8_char_boundary());
                // SAFETY: `index > 0` and `index < self.len()`.
                unsafe { assert_unchecked(self.len() > 1) };
                if self.as_bytes()[1].is_utf8_char_boundary() {
                    1
                } else {
                    0
                }
            }
        }
    }

    #[inline]
    fn ceil_char_boundary_unrolled(&self, index: usize) -> usize {
        if index >= self.len() {
            self.len()
        } else {
            let mut i = index;
            while !self.as_bytes()[i].is_utf8_char_boundary() {
                i += 1;
                if i >= self.len() {
                    break;
                }
            }

            // The character boundary will be within four bytes of the index
            debug_assert!(i <= index + 3);

            i
        }
    }
}

trait U8Ext: Copy {
    fn is_utf8_char_boundary(self) -> bool;
}

impl U8Ext for u8 {
    #[inline]
    fn is_utf8_char_boundary(self) -> bool {
        (self as i8) >= -0x40
    }
}

#[cfg(test)]
#[test]
fn compare_with_std() {
    let s = "Hello, world κόσμος 世界 🌎❗️❗️";
    let r = s.chars().rev().collect::<String>();

    for i in 0..(s.len() + 8) {
        assert_eq!(s.floor_char_boundary(i), s.floor_char_boundary_unrolled(i));
        assert_eq!(s.floor_char_boundary(i), s.floor_char_boundary_mask(i));
        assert_eq!(s.ceil_char_boundary(i), s.ceil_char_boundary_unrolled(i));

        assert_eq!(r.floor_char_boundary(i), r.floor_char_boundary_unrolled(i));
        assert_eq!(r.floor_char_boundary(i), r.floor_char_boundary_mask(i));
        assert_eq!(r.ceil_char_boundary(i), r.ceil_char_boundary_unrolled(i));
    }
}

#[cfg(test)]
#[test]
fn floor_char_boundary_test_adapted_from_std() {
    fn check_many(s: &str, arg: impl IntoIterator<Item = usize>, ret: usize) {
        for idx in arg {
            assert_eq!(
                s.floor_char_boundary_unrolled(idx),
                ret,
                "{:?}.floor_char_boundary_unrolled({:?}) != {:?}",
                s,
                idx,
                ret
            );
            assert_eq!(
                s.floor_char_boundary_mask(idx),
                ret,
                "{:?}.floor_char_boundary_mask({:?}) != {:?}",
                s,
                idx,
                ret
            );
        }
    }

    // edge case
    check_many("", [0, 1, isize::MAX as usize, usize::MAX], 0);

    // basic check
    check_many("x", [0], 0);
    check_many("x", [1, isize::MAX as usize, usize::MAX], 1);

    // 1-byte chars
    check_many("jp", [0], 0);
    check_many("jp", [1], 1);
    check_many("jp", 2..4, 2);

    // 2-byte chars
    check_many("ĵƥ", 0..2, 0);
    check_many("ĵƥ", 2..4, 2);
    check_many("ĵƥ", 4..6, 4);

    // 3-byte chars
    check_many("日本", 0..3, 0);
    check_many("日本", 3..6, 3);
    check_many("日本", 6..8, 6);

    // 4-byte chars
    check_many("🇯🇵", 0..4, 0);
    check_many("🇯🇵", 4..8, 4);
    check_many("🇯🇵", 8..10, 8);

    // anticipate length- and index-based specializations
    let s = "jpĵƥ日本🇯🇵jpĵƥ日本🇯🇵";
    let expected = [
        0, 1, 2, 2, 4, 4, 6, 6, 6, 9, 9, 9, 12, 12, 12, 12, 16, 16, 16, 16, 20, 21, 22, 22, 24, 24,
        26, 26, 26, 29, 29, 29, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40,
    ];
    for (idx, &ret) in expected.iter().enumerate() {
        check_many(s, [idx], ret);
    }
}

#[cfg(test)]
#[test]
fn ceil_char_boundary_test_adapted_from_std() {
    fn check_many(s: &str, arg: impl IntoIterator<Item = usize>, ret: usize) {
        for idx in arg {
            assert_eq!(
                s.ceil_char_boundary_unrolled(idx),
                ret,
                "{:?}.ceil_char_boundary_unrolled({:?}) != {:?}",
                s,
                idx,
                ret
            );
        }
    }

    // edge case
    check_many("", [0], 0);

    // basic check
    check_many("x", [0], 0);
    check_many("x", [1], 1);

    // 1-byte chars
    check_many("jp", [0], 0);
    check_many("jp", [1], 1);
    check_many("jp", [2], 2);

    // 2-byte chars
    check_many("ĵƥ", 0..=0, 0);
    check_many("ĵƥ", 1..=2, 2);
    check_many("ĵƥ", 3..=4, 4);

    // 3-byte chars
    check_many("日本", 0..=0, 0);
    check_many("日本", 1..=3, 3);
    check_many("日本", 4..=6, 6);

    // 4-byte chars
    check_many("🇯🇵", 0..=0, 0);
    check_many("🇯🇵", 1..=4, 4);
    check_many("🇯🇵", 5..=8, 8);

    // above len
    check_many("hello", 5..=10, 5);

    // anticipate length- and index-based specializations
    let s = "jpĵƥ日本🇯🇵jpĵƥ日本🇯🇵";
    let expected = [
        0, 1, 2, 4, 4, 6, 6, 9, 9, 9, 12, 12, 12, 16, 16, 16, 16, 20, 20, 20, 20, 21, 22, 24, 24,
        26, 26, 29, 29, 29, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 40, 40, 40,
    ];
    for (idx, &ret) in expected.iter().enumerate() {
        check_many(s, [idx], ret);
    }
}
