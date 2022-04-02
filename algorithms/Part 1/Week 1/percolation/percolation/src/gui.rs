mod text;
use crate::gui::text::{draw_text_mut, text_size};
use crate::Percolation;
use font_kit::{handle::Handle, source::SystemSource};
use image::{Rgb, RgbImage};
use native_dialog::FileDialog;
use rusttype::{Font, Scale};
use std::fs;

pub const LENGTH: usize = 512;
pub const SCALE: usize = 2; // scales the size of the saved image with respect to frame buffer canvas size
pub const BLOCK_SIZE: f64 = 0.45; // takes value between 0 and 0.5; 0.5 means no gap between blocks, 0 means block size will not be visible
pub const STATUS_HEIGHT: usize = 20;
pub const AREA: usize = LENGTH * LENGTH;
pub const STATUS_AREA: usize = LENGTH * STATUS_HEIGHT;
pub const SCALED_LEN: usize = LENGTH * SCALE;
pub const STATUS_TEXT_HEIGHT: f32 = STATUS_HEIGHT as f32 * 0.7;

pub const fn grayscale(color: u8) -> u32 {
    ((color as u32) << 16) | ((color as u32) << 8) | color as u32
}
pub const fn index(x: usize, y: usize) -> usize {
    y * LENGTH + x
}

pub fn set_font<'a>() -> Font<'a> {
    let font = SystemSource::new()
        .select_by_postscript_name("Arial-BoldMT")
        .unwrap();
    match font {
        Handle::Path { path, font_index } => {
            Font::try_from_vec_and_index(std::fs::read(path).unwrap(), font_index).unwrap()
        }
        Handle::Memory { bytes, font_index } => {
            Font::try_from_vec_and_index(bytes.iter().cloned().collect(), font_index).unwrap()
        }
    }
}

pub fn save_text(text_buf: &String) {
    if let Some(path) = FileDialog::new()
        .set_location("~/Desktop")
        .add_filter("Text File", &["txt"])
        .show_save_single_file()
        .unwrap()
    {
        match path.extension() {
            Some(t) => match t.to_str() {
                Some("txt") => {
                    fs::write(path, &text_buf).unwrap();
                }
                _ => panic!("Unsupported file extension"),
            },
            _ => panic!("No extension in path"),
        };
    }
}

pub fn draw_status_bar(perc: &Percolation, render: bool, font: &Font) -> Vec<u32> {
    let scale: Scale = Scale {
        x: STATUS_TEXT_HEIGHT,
        y: STATUS_TEXT_HEIGHT,
    };
    let nopen;
    let temp;
    let mut left = "";
    let mut right = "";
    if render {
        nopen = perc.number_of_open_sites();
        temp = format!("{} sites opened", &nopen);
        if nopen == 1 {
            left = "1 site opened";
        } else {
            left = temp.as_str();
        }
        if perc.percolates() {
            right = "percolates";
        } else {
            right = "does not percolate";
        }
    }
    let (lefttext_w, lefttext_h) = text_size(scale, &font, &left);
    let (righttext_w, righttext_h) = text_size(scale, &font, &right);
    let mut statusbar = RgbImage::new(LENGTH as u32, STATUS_HEIGHT as u32);
    draw_text_mut(
        &mut statusbar,
        Rgb([255u8, 255u8, 255u8]),
        ((LENGTH as i32 >> 2) - (lefttext_w >> 1)) as i32,
        (STATUS_HEIGHT as i32 - lefttext_h) >> 1,
        scale,
        &font,
        left,
    );
    draw_text_mut(
        &mut statusbar,
        Rgb([255u8, 255u8, 255u8]),
        ((3 * LENGTH as i32) >> 2) - (righttext_w >> 1),
        (STATUS_HEIGHT as i32 - righttext_h) >> 1,
        scale,
        &font,
        right,
    );
    let mut statusbuffer: Vec<u32> = vec![0; STATUS_AREA];
    let mut status_pixelbuf = statusbar.pixels();
    for i in 0..STATUS_AREA {
        let red_channel: u8 = status_pixelbuf.next().unwrap()[1];
        if red_channel != 0 {
            statusbuffer[i] = grayscale(red_channel);
        }
        if (i % LENGTH) == (LENGTH >> 1) || i < LENGTH {
            statusbuffer[i] = 16777215;
        }
    }
    statusbuffer
}

pub fn open() -> Vec<usize> {
    if let Some(path) = FileDialog::new()
        .set_location("~/Desktop")
        .add_filter("Text File", &["txt"])
        .show_open_single_file()
        .unwrap()
    {
        let input_file = fs::read_to_string(path)
            .expect("Invalid file name. If not in current dir include the path");
        return input_file
            .trim()
            .split_whitespace()
            .map(|val| val.parse::<usize>().unwrap())
            .collect::<Vec<_>>();
    }
    vec![]
}

pub fn save_screen(buffer: &mut Vec<u32>) {
    if let Some(path) = FileDialog::new()
        .set_location("~/Desktop")
        .reset_location()
        .add_filter("PNG Image", &["png"])
        .add_filter("JPEG Image", &["jpg", "jpeg"])
        .show_save_single_file()
        .unwrap()
    {
        match path.extension() {
            Some(t) => match t.to_str() {
                Some("jpg" | "jpeg" | "png") => {
                    let img = image::ImageBuffer::from_fn(
                        SCALED_LEN as u32,
                        SCALED_LEN as u32,
                        |x, y| {
                            let buf_idx = buffer[y as usize / SCALE * LENGTH + x as usize / SCALE];
                            image::Rgb([(buf_idx >> 16) as u8, (buf_idx >> 8) as u8, buf_idx as u8])
                        },
                    );
                    img.save(path).unwrap();
                }
                _ => panic!("Unsupported file extension"),
            },
            _ => panic!("No extension in path"),
        };
    }
}

pub fn fill_rect(
    idx: usize,                 //xth column
    idy: usize,                 //yth row
    side_length: usize,         //no. of rows or columns on a side
    half_length: usize,         //length of rect / 2
    color: u32,                 //color of the rectangle
    colorbuffer: &mut Vec<u32>, //mutable color buffer for rendering frame buffer
) {
    let centerx = (LENGTH as f64 * ((idx - 1) as f64 + 0.5) / side_length as f64) as usize;
    let centery = (LENGTH as f64 * ((idy - 1) as f64 + 0.5) / side_length as f64) as usize;
    for x in (centerx - half_length)..(centerx + half_length) {
        for y in (centery - half_length)..(centery + half_length) {
            colorbuffer[index(x, y)] = color;
        }
    }
}
