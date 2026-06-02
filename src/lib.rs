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
pub fn floor_const_std(s: &str) -> usize {
    s.floor_char_boundary(N)
}

#[unsafe(no_mangle)]
pub fn floor_const_unrolled(s: &str) -> usize {
    s.floor_char_boundary_unrolled(N)
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
            if i > 0 && !self.as_bytes()[i].is_utf8_char_boundary() {
                i -= 1;
                if i > 0 && !self.as_bytes()[i].is_utf8_char_boundary() {
                    i -= 1;
                    if !self.as_bytes()[i].is_utf8_char_boundary() {
                        // `&self[0]` will always be a char boundary with valid UTF-8
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
fn test_unrolled() {
    let s = "Hello, 世界❗️❗️";
    let r = s.chars().rev().collect::<String>();

    for i in 0..(s.len() + 8) {
        assert_eq!(s.floor_char_boundary(i), s.floor_char_boundary_unrolled(i));
        assert_eq!(s.ceil_char_boundary(i), s.ceil_char_boundary_unrolled(i));

        assert_eq!(r.floor_char_boundary(i), r.floor_char_boundary_unrolled(i));
        assert_eq!(r.ceil_char_boundary(i), r.ceil_char_boundary_unrolled(i));
    }
}
