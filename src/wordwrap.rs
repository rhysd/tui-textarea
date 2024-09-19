#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum TextWrapMode {
    Width,
    Word,
    WORD,
}

fn fit_col(col: usize, line: &str) -> usize {
    std::cmp::min(col, line.chars().count())
}

fn fit_col_width(col: usize, line: &str, width: usize) -> usize {
    std::cmp::min(col, line.chars().count() % width) + (line.chars().count() / width) * width
}

#[allow(clippy::type_complexity)]
fn get_pos(
    slices: &[((usize, usize), (usize, usize))],
    col: usize,
) -> Option<(usize, usize, usize)> {
    slices
        .iter()
        .enumerate()
        .find(|(i, ((start_char, end_char), _))| {
            if *i == slices.len() - 1 {
                col >= *start_char && col <= *end_char
            } else {
                col >= *start_char && col < *end_char
            }
        })
        .map(|(i, ((start_char, end_char), _))| (i, *start_char, *end_char))
}

pub fn get_cursor_col(
    line: &str,
    col: usize,
    width: usize,
    mode: &TextWrapMode,
) -> Option<(usize, usize)> {
    let slices = compute_slices(line, width, mode);
    let (row, start_char, _) = get_pos(&slices, col)?;
    Some((row, col - start_char))
}

pub fn count_lines(lines: &[String], width: usize, mode: &TextWrapMode) -> usize {
    let mut count = 0;
    for line in lines {
        let slices = compute_slices(line, width, mode);
        count += slices.len();
    }
    count
}

pub fn go_down(
    lines: &[String],
    row: usize,
    col: usize,
    width: usize,
    mode: &TextWrapMode,
) -> Option<(usize, usize)> {
    match mode {
        TextWrapMode::Width => go_down_width(lines, row, col, width),
        TextWrapMode::Word => go_down_words(lines, row, col, width, true),
        TextWrapMode::WORD => go_down_words(lines, row, col, width, false),
    }
}

fn go_down_width(lines: &[String], row: usize, col: usize, width: usize) -> Option<(usize, usize)> {
    let line = &lines[row];
    let slices = line.chars().count() / width;
    if col < slices * width {
        let min = (col + width).min(line.chars().count());
        Some((row, min))
    } else {
        Some((row + 1, fit_col(col % width, lines.get(row + 1)?)))
    }
}

fn go_down_words(
    lines: &[String],
    row: usize,
    col: usize,
    width: usize,
    punctuation: bool,
) -> Option<(usize, usize)> {
    let line = &lines[row];
    let slices = compute_slices_words(line, width, punctuation);

    let (current_line, start_char, end_char) = get_pos(&slices, col)?;
    // let (current_line, start_char, end_char) = slices
    //     .iter()
    //     .enumerate()
    //     .find(|(i, ((start_char, end_char), _))| {
    //         if *i == slices.len() - 1 {
    //             col >= *start_char && col <= *end_char
    //         } else {
    //             col >= *start_char && col < *end_char
    //         }
    //     })
    //     .map(|(i, ((start_char, end_char), _))| (i, *start_char, *end_char))?;

    if current_line == slices.len() - 1 {
        let row = row + 1;
        if row >= lines.len() {
            return None;
        }
        let line = &lines[row];
        let slices = compute_slices_words(line, width, punctuation);
        if slices.is_empty() {
            Some((row, 0))
        } else {
            let ((next_sc, next_ec), _) = slices.first().unwrap();
            let decal = col - start_char;
            let newcol = if next_ec - next_sc > decal {
                next_sc + decal
            } else {
                *next_ec
            };
            Some((row, newcol))
        }
    } else {
        let ((next_sc, next_ec), (_, _)) = slices[current_line + 1];
        let ending_char = if current_line == (slices.len() - 2) {
            1
        } else {
            0
        };
        let decal = col
            .saturating_sub(start_char)
            .min((next_ec - next_sc).saturating_sub(1) + ending_char);
        Some((row, end_char + decal))
    }
}

pub fn go_up(
    lines: &[String],
    row: usize,
    col: usize,
    width: usize,
    mode: &TextWrapMode,
) -> Option<(usize, usize)> {
    match mode {
        TextWrapMode::Width => go_up_width(lines, row, col, width),
        TextWrapMode::Word => go_up_words(lines, row, col, width, true),
        TextWrapMode::WORD => go_up_words(lines, row, col, width, false),
    }
}

fn go_up_width(lines: &[String], row: usize, col: usize, width: usize) -> Option<(usize, usize)> {
    if col >= width {
        Some((row, col - width))
    } else {
        let row = row.checked_sub(1)?;
        Some((row, fit_col_width(col, &lines[row], width)))
    }
}

fn go_up_words(
    lines: &[String],
    row: usize,
    col: usize,
    width: usize,
    punctuation: bool,
) -> Option<(usize, usize)> {
    let line = &lines[row];
    let slices = compute_slices_words(line, width, punctuation);

    let (current_line, start_char, _) = get_pos(&slices, col)?;
    // let (current_line, start_char) = slices
    //     .iter()
    //     .enumerate()
    //     .find(|(i, ((start_char, end_char), _))| {
    //         if *i == slices.len() - 1 {
    //             col >= *start_char && col <= *end_char
    //         } else {
    //             col >= *start_char && col < *end_char
    //         }
    //     })
    //     .map(|(i, ((start_char, _), _))| (i, *start_char))?;

    if current_line == 0 {
        let row = row.checked_sub(1)?;
        let line = &lines[row];
        let slices = compute_slices_words(line, width, punctuation);
        if slices.is_empty() {
            Some((row, 0))
        } else {
            let ((prev_sc, prev_ec), _) = slices.last().unwrap();
            let decal = col - start_char;
            let newcol = if prev_ec - prev_sc > decal {
                prev_sc + decal
            } else {
                *prev_ec
            };
            Some((row, newcol))
        }
    } else {
        let ((prev_sc, prev_ec), (_, _)) = slices[current_line - 1];
        let decal = (prev_ec - prev_sc).saturating_sub(col - start_char).max(1);
        Some((row, start_char - decal))
    }
}

pub fn compute_slices(
    line: &str,
    width: usize,
    mode: &TextWrapMode,
) -> Vec<((usize, usize), (usize, usize))> {
    match mode {
        TextWrapMode::Width => compute_slices_width(line, width),
        TextWrapMode::Word => compute_slices_words(line, width, true),
        TextWrapMode::WORD => compute_slices_words(line, width, false),
    }
}

fn compute_slices_width(line: &str, width: usize) -> Vec<((usize, usize), (usize, usize))> {
    let full_lines_count = line.chars().count() / width;

    let mut slices = vec![];
    for i in 0..full_lines_count {
        let offset = i * width;
        let (first, _) = line.char_indices().skip(offset).take(1).last().unwrap();
        let (last, _) = line
            .char_indices()
            .skip(offset + width)
            .take(1)
            .last()
            .unwrap_or((line.len(), ' '));
        slices.push(((offset, offset + width), (first, last)));
    }
    if line.is_empty() {
        slices.push(((0, 0), (0, 0)));
    } else if line.chars().count() % width != 0 {
        let offset = full_lines_count * width;
        let (first, _) = line.char_indices().skip(offset).take(1).last().unwrap();
        slices.push(((offset, line.chars().count()), (first, line.len())));
    } else {
        let c = line.chars().count();
        let l = line.len();
        slices.push(((c, c), (l, l)));
    }
    slices
}

#[allow(clippy::type_complexity)]
fn compute_next(
    line: &str,
    start_char: usize,
    start_byte: usize,
    width: usize,
    parts: &mut Vec<((usize, usize), (usize, usize))>,
    punctuation: bool,
) {
    let nb_chars = line.chars().count();
    let nb_bytes = line.len();

    let mut end_char = start_char + width;

    if start_char >= nb_chars {
        parts.push(((0, 0), (0, 0)));
    } else if end_char >= nb_chars {
        parts.push(((start_char, nb_chars), (start_byte, nb_bytes)));
    } else {
        let start_byte = line.char_indices().nth(start_char).map(|(i, _)| i).unwrap();
        let mut end_byte = line.char_indices().nth(end_char).map(|(i, _)| i).unwrap();

        let part = &line[start_byte..end_byte];

        let new_end_char = part
            .char_indices()
            .rev()
            .enumerate()
            .find(|(_, (_, c))| {
                if punctuation {
                    c.is_ascii_punctuation() || c.is_whitespace()
                } else {
                    c.is_whitespace()
                }
            })
            .map(|(pos_char, _)| width - pos_char + start_char)
            .unwrap_or(end_char);

        if new_end_char != end_char {
            end_char = new_end_char;
            end_byte = line.char_indices().nth(end_char).map(|(i, _)| i).unwrap();
        }

        parts.push(((start_char, end_char), (start_byte, end_byte)));

        compute_next(line, end_char, end_byte, width, parts, punctuation);
    }
}

fn compute_slices_words(
    line: &str,
    width: usize,
    punctuation: bool,
) -> Vec<((usize, usize), (usize, usize))> {
    let mut slices = vec![];
    compute_next(line, 0, 0, width, &mut slices, punctuation);
    slices
}
