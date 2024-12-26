use std::{process::Command, thread, time::Duration};

use bit_set::BitSet;
use colored::{Color, ColoredString, Colorize};
use rand::{rngs::ThreadRng, thread_rng, Rng};

const BROWN: Color = Color::TrueColor {
    r: 139,
    g: 69,
    b: 19,
};

trait StringWidth {
    fn width(&self) -> usize;
}

impl StringWidth for String {
    fn width(&self) -> usize {
        return self
            .chars()
            .map(|c| {
                if c.is_ascii() {
                    return 1;
                } else {
                    return 2;
                }
            })
            .sum();
    }
}

impl StringWidth for ColoredString {
    fn width(&self) -> usize {
        return self.input.width();
    }
}

#[derive(Clone)]
enum Content {
    Transparent,
    ColoredString { s: ColoredString },
    Compensate,
}

trait Frame {
    fn update(&mut self, screen_width: usize, screen_height: usize);
    fn get_content(&mut self, x: usize, y: usize) -> Content;
}

struct SnowFrame {
    thread_rng: ThreadRng,
    frame_width: usize,
    frame_height: usize,
    cursor: usize,
    snows_row: Vec<BitSet>,
}

impl Default for SnowFrame {
    fn default() -> Self {
        return SnowFrame {
            thread_rng: thread_rng(),
            frame_width: 0,
            frame_height: 0,
            cursor: 0,
            snows_row: Vec::new(),
        };
    }
}

impl Frame for SnowFrame {
    fn update(&mut self, screen_width: usize, screen_height: usize) {
        if self.frame_width != screen_width || self.frame_height != screen_height {
            self.frame_width = screen_width;
            self.frame_height = screen_height;
            self.cursor = 0;
            self.snows_row = vec![BitSet::with_capacity(screen_width); screen_height];
        } else {
            self.cursor = (self.cursor + screen_height - 1) % screen_height;
        }

        let snows = &mut self.snows_row[self.cursor];
        snows.clear();

        for i in 0..self.frame_width {
            if self.thread_rng.gen_range(0..=20) == 0 {
                snows.insert(i);
            }
        }
    }

    fn get_content(&mut self, x: usize, y: usize) -> Content {
        let y = (self.cursor + y) % self.frame_height;
        if self.snows_row[y].contains(x) {
            return Content::ColoredString { s: "o".white() };
        } else {
            return Content::Transparent;
        }
    }
}

struct ChristmasTreeFrame {
    thread_rng: ThreadRng,
    frame_width: usize,
    frame_height: usize,
}

impl ChristmasTreeFrame {
    fn get_leaf_color(&mut self) -> Color {
        return match self.thread_rng.gen_range(0..=5) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Yellow,
            3 => Color::Blue,
            4 => Color::Magenta,
            5 => Color::Cyan,
            _ => Color::White,
        };
    }

    fn get_leaf(&mut self) -> ColoredString {
        return match self.thread_rng.gen_range(0..=10) {
            0 => "o".color(self.get_leaf_color()),
            _ => "*".green(),
        };
    }
}

impl Default for ChristmasTreeFrame {
    fn default() -> Self {
        return ChristmasTreeFrame {
            thread_rng: thread_rng(),
            frame_width: 0,
            frame_height: 0,
        };
    }
}

impl Frame for ChristmasTreeFrame {
    fn update(&mut self, screen_width: usize, screen_height: usize) {
        if self.frame_width != screen_width || self.frame_height != screen_height {
            self.frame_width = screen_width;
            self.frame_height = screen_height;
        }
    }

    fn get_content(&mut self, x: usize, y: usize) -> Content {
        const HEIGHT: usize = 14;
        // christmas tree only shows on the middle of 14 rows
        let y_offset = (self.frame_height - HEIGHT) / 2;
        if y < y_offset || y >= y_offset + HEIGHT {
            return Content::Transparent;
        }

        // leaf part
        const LEAF_HEIGHT: usize = 10;
        if y - y_offset < LEAF_HEIGHT {
            let leaf_width = 2 * (y - y_offset) + 1;
            let leaf_offset = (self.frame_width - leaf_width) / 2;
            if x < leaf_offset || x >= leaf_offset + leaf_width {
                return Content::Transparent;
            } else {
                return Content::ColoredString { s: self.get_leaf() };
            }
        }

        // trunk part
        const TRUNK_HEIGHT: usize = 2;
        if y - y_offset - LEAF_HEIGHT < TRUNK_HEIGHT {
            let trunk = "mWm".to_owned();
            let trunk_vec = string_to_content_vec(&trunk, BROWN);
            let trunk_width = 3;
            let trunk_offset = (self.frame_width - trunk_width) / 2;
            if x < trunk_offset || x >= trunk_offset + trunk_width {
                return Content::Transparent;
            } else {
                return trunk_vec[x - trunk_offset].clone();
            }
        }

        // blank part
        const BLANK_HEIGHT: usize = 1;
        if y - y_offset - LEAF_HEIGHT - TRUNK_HEIGHT < BLANK_HEIGHT {
            return Content::Transparent;
        }

        // blessing part
        const BLESSING_HEIGHT: usize = 1;
        if y - y_offset - LEAF_HEIGHT - TRUNK_HEIGHT - BLANK_HEIGHT < BLESSING_HEIGHT {
            let blessing = "2024 聖誕快樂".to_owned();
            let blessing_vec = string_to_content_vec(&blessing, Color::Red);
            let blessing_width = blessing.width();
            let blessing_offset = (self.frame_width - blessing_width) / 2;
            if x < blessing_offset || x >= blessing_offset + blessing_width {
                return Content::Transparent;
            } else {
                return blessing_vec[x - blessing_offset].clone();
            }
        }

        return Content::Transparent;
    }
}

struct Printer {
    screen_width: usize,
    screen_height: usize,
    frames: Vec<Box<dyn Frame>>,
}

impl Printer {
    fn new(frames: Vec<Box<dyn Frame>>) -> Self {
        return Printer {
            screen_width: 0,
            screen_height: 0,
            frames,
        };
    }

    fn update(&mut self) {
        let (screen_width, screen_height) = term_size::dimensions().unwrap();
        self.screen_width = screen_width;
        self.screen_height = screen_height;

        for frame in self.frames.iter_mut() {
            frame.update(screen_width, screen_height);
        }
    }

    fn clear(&self) {
        if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
        } else {
            Command::new("clear").status().unwrap();
        }
    }

    fn print(&mut self) {
        let contents = (0..self.screen_height)
            .into_iter()
            .map(|y| {
                let mut row_strings = String::new();

                let mut x = 0;
                while x < self.screen_width {
                    let content = self
                        .frames
                        .iter_mut()
                        .map(|frame| frame.get_content(x, y))
                        .find(|content| match content {
                            Content::Transparent | Content::Compensate => false,
                            _ => true,
                        });

                    match content {
                        Some(Content::ColoredString { s }) => {
                            row_strings.push_str(&s.to_string());
                            x += s.width() - 1;
                        }
                        _ => {
                            row_strings.push(' ');
                        }
                    }

                    x += 1;
                }

                return row_strings;
            })
            .collect::<Vec<String>>()
            .join("\n");

        print!("{}", contents);
    }
}

fn string_to_content_vec(s: &str, color: Color) -> Vec<Content> {
    let string_width = s.to_owned().width();
    let mut content_vec = Vec::<Content>::with_capacity(string_width);

    s.chars().into_iter().for_each(|c| {
        content_vec.push(Content::ColoredString {
            s: c.to_string().color(color),
        });

        if !c.is_ascii() {
            content_vec.push(Content::Compensate);
        }
    });

    return content_vec;
}

fn main() {
    let snow_frame = Box::new(SnowFrame::default());
    let christmas_tree_frame = Box::new(ChristmasTreeFrame::default());
    let mut printer = Printer::new(vec![christmas_tree_frame, snow_frame]);

    loop {
        printer.update();
        printer.clear();
        printer.print();
        thread::sleep(Duration::from_millis(1000));
    }
}
