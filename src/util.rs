pub fn spaces(size: u8) -> &'static str {
    const SPACES: &str = "                                                                                                                                                                                                                                                                ";
    &SPACES[..size as usize]
}

pub fn num_digits(i: usize) -> u8 {
    f64::log10(i as f64) as u8 + 1
}

/// Calculate number of rows for a wrapped line
pub fn line_rows(line: &String, wrap_width: u16, has_lnum: bool, num_lines: usize) -> u16 {
    let lnum_span_len = if has_lnum {
        // Longest line number plus space on each side
        num_digits(num_lines) + 2
    } else {
        0
    };

    let words = line.split_whitespace();
    let mut current_line_len = lnum_span_len;
    let mut wraps = 0;

    for word in words {
        let word_len = word.chars().count() as u8;

        if (current_line_len + word_len + 1) as u16 > wrap_width {
            wraps += 1;
            current_line_len = 0;
        }

        current_line_len += word_len + 1;
    }

    // Add 1 to account for the last line
    (wraps + 1).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_line_rows_test(
        line: &str,
        wrap_width: u16,
        has_lnum: bool,
        num_lines: usize,
        expected: u16,
    ) {
        let line = line.to_string();
        let result = line_rows(&line, wrap_width, has_lnum, num_lines);
        assert_eq!(
            result, expected,
            "with string: '{}', width: {}, lnum: {}, num_lines: {}",
            line, wrap_width, has_lnum, num_lines
        );
    }

    #[test]
    fn test_empty_line() {
        run_line_rows_test("", 1, false, 1, 1);
        run_line_rows_test("", 10, true, 10, 1);
    }

    #[test]
    fn test_no_wrapping() {
        run_line_rows_test("Hello, world!", 13, false, 1, 1);
        // _10_Hello, world!
        run_line_rows_test("Hello, world!", 17, true, 10, 1);
    }

    #[test]
    fn test_wrapping() {
        run_line_rows_test(
            "A long line that should wrap at least once.",
            20,
            false,
            1,
            3,
        );
        run_line_rows_test(
            "A long line that should wrap at least once.",
            20,
            true,
            10,
            3,
        );
    }

    #[test]
    fn test_long_word_wrapping() {
        // This line
        // has a
        // longwordth
        // atwraps.
        run_line_rows_test("This line has a longwordthatwraps.", 10, false, 1, 4);
        // _10_This
        // line has a
        // longwordth
        // atwraps.
        run_line_rows_test("This line has a longwordthatwraps.", 10, true, 10, 4);
        // _10_This
        // line has a
        // longwordth
        // atwrapsmor
        // e.
        run_line_rows_test("This line has a longwordthatwrapsmore.", 10, true, 10, 5);
    }

    #[test]
    fn test_long_word_overflow() {
        // This line
        // has a
        // longwordth
        // atoverflow
        // s.
        run_line_rows_test("This line has a longwordthatoverflows.", 10, false, 1, 5);
        // _100_ This
        // line has a
        // longwordth
        // atoverflow
        // s.
        run_line_rows_test("This line has a longwordthatoverflows.", 10, true, 100, 5);
        // _100000_
        // This line
        // has a
        // longwordth
        // atoverflow
        // s.
        run_line_rows_test(
            "This line has a longwordthatoverflows.",
            10,
            true,
            100000,
            6,
        );
    }

    #[test]
    fn test_line_numbers() {
        // _1_Word
        // Word Word
        // Word
        run_line_rows_test("Word Word Word Word", 10, true, 1, 3);
        // _1000_Word
        // Word Word
        // Word
        run_line_rows_test("Word Word Word Word", 10, true, 1000, 3);
        // _10000_
        // Word Word
        // Word Word
        run_line_rows_test("Word Word Word Word", 10, true, 10000, 3);

        // _1_ Longer
        run_line_rows_test("Longer", 10, true, 1, 1);
        // _10_
        // Longer
        run_line_rows_test("Longer", 10, true, 10, 2);
    }
}
