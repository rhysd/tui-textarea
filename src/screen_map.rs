use unicode_width::UnicodeWidthChar;

use crate::{
    cursor::{DataCursor, ScreenCursor},
    history::EditKind,
    word::find_word_start_backward,
    TextArea, WrapMode,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LinePtr {
    pub data_line: usize,    // parent data line index
    pub byte_offset: usize,  // start of slice line in data line
    pub byte_length: usize,  // length of slice in data line
    pub data_offset: usize,  // offset of slice in parent data line
    pub screen_width: usize, // space occupied on screen by this line
    pub lp_num: usize,       // this is the nth lp for a given data line
    pub lp_count: usize,     // this is the number of lps for a given data line
    pub last: bool,          // this is the last lp for a given data line
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DataLine {
    first_screen_line: usize, // pointer to first LinePtr for this data line
    pure_ascii: bool,         // line only has ascii and no hard tabs
}

// lifted from rust std source
pub(crate) fn is_utf8_char_boundary(b: u8) -> bool {
    (b as i8) >= -0x40
}

impl<'a> TextArea<'a> {
    pub(crate) fn screen_line_width(&self, line: usize) -> usize {
        // the spce occuupied by a screen line , in 1 byte chars
        self.screen_lines.borrow()[line].screen_width
    }
    pub(crate) fn screen_lines_count(&self) -> usize {
        self.screen_lines.borrow().len()
    }
    pub(crate) fn screen_cursor_in_bounds(&self, sc: ScreenCursor) -> bool {
        let screen_lines = self.screen_lines.borrow();
        trace!("screen_cursor_in_bounds {:?} ", sc);
        if sc.row >= screen_lines.len() {
            trace!("screen_cursor_in_bounds false row {}", screen_lines.len());
            return false;
        }
        let lp = &screen_lines[sc.row];
        if sc.col >= lp.screen_width {
            trace!("screen_cursor_in_bounds false col {}", lp.screen_width);
            return false;
        }
        trace!("screen_cursor_in_bounds true");
        true
    }
    pub(crate) fn screen_to_array(&self, screen: ScreenCursor) -> DataCursor {
        trace!("screen_to_array start {:?} ", screen);
        if screen.dc.is_some() {
            trace!("Fast out");
            //     return screen.dc.unwrap();
        }
        let screen_lines = self.screen_lines.borrow();
        let screen_line = &screen_lines[screen.row];
        let data_pointer = &(*self.data_pointers.borrow())[screen_line.data_line];
        let data_idx = screen_line.data_line;

        // if we are in pure ascii mode then and not wrapping
        // then sc === dc
        let fasdc = if self.wrap_mode == WrapMode::None && data_pointer.pure_ascii {
            trace!("Fastish out");
            Some(DataCursor(data_idx, screen.col))
        } else {
            None
        };
        let screen_lines = self.screen_lines.borrow();
        let screen_line = &screen_lines[screen.row];
        let data_idx = screen_line.data_line;
        let mut off = 0;

        let last_lp_line = &screen_lines[screen.row];
        trace!("last_lp_line {:?} ", last_lp_line);
        let mut chidx = 0;
        let slice = &self.lines[data_idx][last_lp_line.byte_offset..];

        for (_i, c) in slice.char_indices() {
            if off == screen.col {
                break;
            };

            if off > screen.col {
                // this happens if somebody asks about a char
                // at r.c and its not on a char boundary
                // so we back up one char
                chidx -= 1;
                break;
            }
            chidx += 1;
            off += UnicodeWidthChar::width(c).unwrap_or(0);
        }
        //assert_eq!(off, screen.col);
        let off = last_lp_line.data_offset + chidx;
        let ret = DataCursor(screen_line.data_line, off);
        trace!("screen_to_array {:?} {:?}", screen, ret);
        if fasdc.is_some() {
            assert_eq!(fasdc.unwrap(), ret);
        }
        ret
    }
    pub(crate) fn array_to_screen(&self, array: DataCursor) -> ScreenCursor {
        trace!("array_to_screen start {:?} ", array);
        // if we are in pure ascii mode then and not wrapping
        // then sc === dc
        let pure = self.data_pointers.borrow()[array.0].pure_ascii;
        let fastsc = if self.wrap_mode == WrapMode::None && pure {
            trace!("Fastish out");
            Some(ScreenCursor {
                row: array.0,
                col: array.1,
                char: Some(self.lines[array.0].chars().nth(array.1).unwrap()),
                dc: Some(array),
            })
        } else {
            None
        };
        let first_screen_line_idx = self.data_pointers.borrow()[array.0].first_screen_line;
        let screen_lines = self.screen_lines.borrow();
        let data_line = &self.lines[array.0];
        let mut found_idx = first_screen_line_idx;
        for lp in &screen_lines[first_screen_line_idx..] {
            if lp.data_line != array.0 || lp.data_offset > array.1 {
                break;
            }
            found_idx += 1;
        }
        let found_lp = &screen_lines[found_idx - 1];
        let mut width = 0;
        // let mut idx = 0;
        let mut ch = None;
        trace!("found_lp {:?} {}", found_lp, data_line);
        for (idx, (_i, c)) in data_line[found_lp.byte_offset..].char_indices().enumerate() {
            if idx == array.1 - found_lp.data_offset {
                ch = Some(c);
                break;
            }

            width += UnicodeWidthChar::width(c).unwrap_or(0);
        }

        let ret = ScreenCursor {
            row: found_idx - 1,
            col: width,
            char: ch,
            dc: Some(array),
        };
        trace!("array_to_screen {:?} {:?}", array, ret);
        if fastsc.is_some() {
            assert_eq!(fastsc.unwrap(), ret);
        }
        ret
    }

    fn analyze_line(line: &str) -> (Vec<(usize, usize, char, usize)>, bool) {
        let mut v = Vec::new();
        let mut pure_ascii = false;
        for (i, c) in line.char_indices() {
            let utf8 = c.len_utf8();
            let width = UnicodeWidthChar::width(c).unwrap_or(0);
            if width > 1 || !utf8 != 1 || c == '\t' {
                pure_ascii = false;
            }
            v.push((utf8, width, c, i));
        }
        (v, pure_ascii)
    }
    fn chop_line(&self, line: &str, width: usize, line_num: usize) -> (Vec<LinePtr>, bool) {
        let width = if self.wrap_mode == WrapMode::None {
            usize::MAX
        } else {
            width
        };
        let mut start = 0;
        let mut lps = Vec::new();
        let (al, pure) = Self::analyze_line(line);
        //let data_lines = al.iter().map(|(u, _, _, _)| *u as u8).collect::<Vec<_>>();
        let on_screen_len = al.iter().fold(0, |acc, (_, w, _, _)| acc + w);
        trace!("chop_line {} {} {}", on_screen_len, width, pure);
        let mut chidx = 0;
        let mut scwidth = 0;
        let mut dataoff = 0;

        while chidx < al.len() {
            let alc = &al[chidx];
            dataoff = alc.3;
            if scwidth + alc.1 > width {
                trace!("chop_line {} {} {}", scwidth, start, chidx);
                // reached part of line that overflows screen width
                if let Some(off) = find_word_start_backward(line, chidx) {
                    trace!(
                        "find back({:?}, chidx-> {:?} returns {} -> {:?}",
                        al[start],
                        al[chidx],
                        off,
                        al[off]
                    );

                    if off == 0 || off - start < 2 {
                        // no word to back up to, just chop the line
                        let lp = LinePtr {
                            data_line: line_num,
                            byte_offset: al[start].3,
                            byte_length: al[chidx].3 - al[start].3,
                            data_offset: start,
                            screen_width: scwidth,
                            last: false,
                            lp_num: lps.len(),
                            lp_count: 0,
                        };
                        trace!("lp {:?} ", lp);
                        debug_assert!(is_utf8_char_boundary(line.as_bytes()[al[start].3]));
                        start = chidx;
                        lps.push(lp);
                    } else {
                        // off returns start of prior word
                        // back track screen width the start of word
                        trace!("backtrack {:?} {} ", chidx, off);
                        for i in off..chidx + 1 {
                            trace!("backtrack {:?} {} ", al[i], scwidth);
                            scwidth -= al[i].1;
                        }
                        trace!("st {:?} scan {:?} ", al[off], al[start]);
                        let lp = LinePtr {
                            data_line: line_num,
                            byte_offset: al[start].3,
                            byte_length: al[off].3 - al[start].3,
                            data_offset: start,
                            screen_width: scwidth,
                            last: false,
                            lp_num: lps.len(),
                            lp_count: 0,
                        };
                        trace!("lp {:?} ", lp);
                        debug_assert!(is_utf8_char_boundary(line.as_bytes()[al[start].3]));
                        debug_assert!(is_utf8_char_boundary(
                            line.as_bytes()[al[start].3 + lp.byte_length]
                        ));
                        start = off;
                        chidx = off;
                        lps.push(lp);
                    }
                } else {
                    break;
                }
                scwidth = 0;
            } else {
                chidx += 1;
                scwidth += alc.1;
            }
        }
        let lp = if chidx == 0 {
            // the whole line fits on the screen
            LinePtr {
                data_line: line_num,
                byte_offset: 0,
                byte_length: 0,
                data_offset: 0,
                screen_width: scwidth,
                last: true,
                lp_num: lps.len(),
                lp_count: 0,
            }
        } else {
            // the last peice of the line
            assert!(is_utf8_char_boundary(line.as_bytes()[al[start].3]));
            LinePtr {
                data_line: line_num,
                byte_offset: al[start].3,
                byte_length: line.len() - al[start].3,
                data_offset: start,
                screen_width: scwidth,
                last: true,
                lp_num: lps.len(),
                lp_count: 0,
            }
        };
        lps.push(lp);
        if scwidth == width {
            // we exactly hit the width, need a new empty next screen line
            // becuase the cursor moves to the next line
            let lpc = lps.len() - 1;
            lps[lpc].last = false;
            let lp = LinePtr {
                data_line: line_num,
                byte_offset: lps[lpc].byte_length + lps[lpc].byte_offset,
                byte_length: 0,
                data_offset: dataoff + 1,
                screen_width: 0,
                last: true,
                lp_num: lps.len(),
                lp_count: 0,
            };
            lps.push(lp);
        }
        let lp_count = lps.len();
        for i in 0..lps.len() {
            let lp = &mut lps[i];
            lp.lp_count = lp_count;
        }

        (lps, pure)
    }

    fn get_width(&self) -> usize {
        self.area.get().width as usize
    }

    pub(crate) fn screen_map_load(&self) {
        {
            let mut screen_lines = self.screen_lines.borrow_mut();
            screen_lines.clear();
            let mut data_pointers = self.data_pointers.borrow_mut();
            data_pointers.clear();
            let width = self.get_width();
            trace!("load {} {}", self.lines.len(), width);
            for (line_num, line) in self.lines.iter().enumerate() {
                let (cl, pure) = self.chop_line(line, width, line_num);
                data_pointers.push(DataLine {
                    first_screen_line: screen_lines.len(),
                    pure_ascii: pure,
                });
                screen_lines.extend(cl);
            }
        }
        self.dump();
    }

    // the follwing functions are perf optimizations
    // rather than recalculating the screen table from scratch
    // common operations update it incrementatlly

    pub(crate) fn update_screen_map(&mut self, kind: &EditKind, cursor_before: DataCursor) {
        match kind {
            EditKind::InsertChar(_, _) => self.update_line(cursor_before.0),

            EditKind::DeleteChar(_, _) => self.update_line(cursor_before.0),

            EditKind::InsertNewline(_) => self.insert_line(cursor_before.0),
            EditKind::DeleteNewline(_) => self.delete_line(cursor_before.0),
            _ => self.screen_map_load(),
        }
    }

    fn update_line(&mut self, row: usize) {
        // update table when a line is changed

        let line = &self.lines[row];
        let (cl, _pure) = self.chop_line(line, self.get_width(), row);
        let line_start = self.data_pointers.get_mut()[row].first_screen_line;
        let screen_lines = self.screen_lines.get_mut();

        let curr_first_lp = &screen_lines[line_start];
        let lp_count = curr_first_lp.lp_count;
        let cl_count = cl.len();
        screen_lines.splice(line_start..line_start + lp_count, cl);

        let delta = lp_count as i32 - cl_count as i32;

        for i in row + 1..self.data_pointers.get_mut().len() {
            let dp = &mut self.data_pointers.get_mut()[i];
            dp.first_screen_line = (dp.first_screen_line as i32 - delta) as usize;
        }
        //self.verify_update();
    }
    fn delete_line(&mut self, row: usize) {
        //wip - update tablasdfe when line is deleted

        trace!("delete_line {}", row);
        assert!(row != 0);
        let line_start = self.data_pointers.get_mut()[row].first_screen_line;
        let screen_lines = self.screen_lines.get_mut();
        self.data_pointers.get_mut().remove(row);
        let curr_first_lp = &screen_lines[line_start];
        assert!(curr_first_lp.lp_count == 1);
        screen_lines.remove(line_start);
        for i in line_start..screen_lines.len() {
            let lp = &mut screen_lines[i];
            lp.data_line -= 1;
        }
        for i in row..self.data_pointers.get_mut().len() {
            let dp = &mut self.data_pointers.get_mut()[i];
            dp.first_screen_line -= 1;
        }
        self.update_line(row - 1);
        // self.verify_update();
    }

    fn insert_line(&mut self, row: usize) {
        // wip - update table when line is inserted

        trace!("insert_line {}", row);

        let row = row + 1;
        let line = &self.lines[row];
        let (cl, pure) = self.chop_line(line, self.get_width(), row);
        let screen_lines = self.screen_lines.get_mut();
        let line_start = if row == 0 {
            0
        } else {
            let x = self.data_pointers.get_mut()[row - 1].first_screen_line;
            let curr_first_lp = &screen_lines[x];
            x + curr_first_lp.lp_count
        };
        for i in line_start..screen_lines.len() {
            let lp = &mut screen_lines[i];
            lp.data_line += 1;
        }
        for i in row..self.data_pointers.get_mut().len() {
            let dp = &mut self.data_pointers.get_mut()[i];
            dp.first_screen_line += 1;
        }
        screen_lines.splice(line_start..line_start, cl);
        self.data_pointers.get_mut().insert(
            row,
            DataLine {
                first_screen_line: line_start,
                pure_ascii: pure,
            },
        );
        self.update_line(row - 1);
        //  self.verify_update();
    }

    fn _verify_update(&mut self) {
        self.dump();
        let save_da = self.data_pointers.borrow().clone();
        let save_lp = self.screen_lines.borrow().clone();
        self.screen_map_load();
        self.dump();
        assert_eq!(save_da, *self.data_pointers.borrow());
        assert_eq!(save_lp, *self.screen_lines.borrow());
    }

    pub(crate) fn data_cursor(&self, sc: ScreenCursor) -> DataCursor {
        sc.to_array_cursor(self)
    }
    pub fn viewport(&self) -> (u16, u16, u16, u16) {
        self.viewport.position()
    }
    pub(crate) fn increment_screen_cursor(&self, sc: ScreenCursor) -> ScreenCursor {
        let dc = sc.to_array_cursor(self);
        let char = self.lines[dc.0].chars().nth(dc.1).unwrap();
        let width = UnicodeWidthChar::width(char).unwrap_or(0);
        ScreenCursor {
            col: sc.col + width,
            ..sc
        }
    }
    pub(crate) fn decrement_screen_cursor(&self, sc: ScreenCursor) -> ScreenCursor {
        let dc = sc.to_array_cursor(self);
        let char = self.lines[dc.0].chars().nth(dc.1 - 1).unwrap();
        let width = UnicodeWidthChar::width(char).unwrap_or(0);
        ScreenCursor {
            col: sc.col - width,
            ..sc
        }
    }
    // pub(crate) fn char_at_screen_cursor(&self, sc: ScreenCursor) -> Option<char> {
    //     let dc = sc.to_array_cursor(&self);
    //     let char = self.lines[dc.0].chars().nth(dc.1);
    //     char
    // }
    pub(crate) fn char_at_array_cursor(&self, dc: DataCursor) -> Option<char> {
        let char = self.lines[dc.0].chars().nth(dc.1);
        char
    }
    fn dump(&self) {
        let screen_lines = self.screen_lines.borrow();
        for lp in &(*screen_lines) {
            trace!("{:?} ", lp);
        }

        for i in 0..self.lines.len() {
            trace!(
                "data pointer {} {:?} {}",
                i,
                self.data_pointers.borrow()[i],
                self.lines[i]
            );
        }
    }
}
#[cfg(test)]
mod test {
    use crate::ratatui::layout::Rect;

    use super::*;
    fn round_trip_a(dc: DataCursor, ta: &TextArea) {
        trace!("round_trip_a {:?}", dc);
        let sc = ta.array_to_screen(dc);
        assert_eq!(ta.screen_to_array(sc), dc);
    }
    fn round_trip_s(sc: ScreenCursor, ta: &TextArea) {
        trace!("round_trip_s {:?}", sc);
        let dc = ta.screen_to_array(sc);
        let retsc = ta.array_to_screen(dc);
        assert_eq!(retsc.row, sc.row);
        assert_eq!(retsc.col, sc.col);
    }

    fn smash_sm(lines: &Vec<String>) {
        let tta = TextArea::from(lines);
        tta.area.set(Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 100,
        });
        tta.screen_map_load();
        let screen_lines = &*tta.screen_lines.borrow();

        for lp in screen_lines {
            trace!("{:?} ", lp);
        }
        for lp in screen_lines {
            let slice =
                lines[lp.data_line][lp.byte_offset..lp.byte_length + lp.byte_offset].to_string();
            trace!("{:?} ", slice);
        }

        for dp in tta.data_pointers.borrow().iter() {
            trace!("{:?} ", dp);
        }
        round_trip_a(DataCursor(0, 5), &tta);
        for i in 0..lines.len() {
            round_trip_a(DataCursor(i, 0), &tta);
            //    round_trip_a(DataCursor(i, lines[i].chars().count() - 1), &tta);
            //  round_trip_a(DataCursor(i, lines[i].chars().count() / 2), &tta);
            for j in 0..lines[i].chars().count() {
                round_trip_a(DataCursor(i, j), &tta);
            }
        }
        for i in 0..screen_lines.len() {
            round_trip_s(
                ScreenCursor {
                    row: i,
                    col: 0,
                    char: None,
                    dc: None,
                },
                &tta,
            );
            //     round_trip_s(ScreenCursor(i, screen_lines[i].byte_length - 1), &tta);
            //     round_trip_s(ScreenCursor(i, screen_lines[i].byte_length / 2), &tta);
        }
    }
    #[test]
    fn test_wrap_ascii() {
        let lorem = ["Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.  ",
    "Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.",
     " It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum"];
        // let lines = vec!("Lorem ipsum dolor sit amet consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore");
        let lines = lorem.iter().map(|s| s.to_string()).collect::<Vec<String>>();

        smash_sm(&lines);
    }
    #[test]
    fn test_wrap_chinese() {
        let multi = ["棵動道吧鼻帶牙村陽風童時抄或和現至至游學：黃朱苗綠音急石消看造貝同鳥再免告二士，很這員冰；飛造孝給以士工香浪、朱的化故水固聽路鴨工根來流胡，怎科麻造菜忍吉？共行發經信是，書視松北吉不"];
        let lines = multi.iter().map(|s| s.to_string()).collect::<Vec<String>>();

        smash_sm(&lines);
    }
    #[test]
    fn test_russian() {
        let multi = ["Лорем ипсум долор сит амет, усу алтера путант цу. Вел ут еяуидем оффициис сцрипторем. Вивендум номинати сигниферумяуе не вим, омниум мнесарчум ат вим, ад утинам рецтеяуе репудиандае еос. Нец еи цлита бландит, сеа постеа модератиус персеяуерис ет. Еам ин веро пауло епицури. Цум еа порро сингулис, молестиае яуаерендум нец ан. Ид меа аудиам ассентиор, еи вис тамяуам сцрипторем.",
        "Воцибус нусяуам сеа ат. Темпор цонсецтетуер еам ат. Идяуе сцрибентур ех меа. Ин неморе лабитур сед."];
        let lines = multi.iter().map(|s| s.to_string()).collect::<Vec<String>>();

        smash_sm(&lines);
    }
}
