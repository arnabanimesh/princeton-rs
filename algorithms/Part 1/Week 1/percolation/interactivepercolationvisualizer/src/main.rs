#![windows_subsystem = "windows"]

use minifb::{
    Icon, InputCallback, Key, Menu, MouseButton, MouseMode, Window, WindowOptions, MENU_KEY_CTRL,
};
use percolation::{gui::*, Percolation};
use std::{env::args, fmt::Write};

const SAVE_SCREEN: usize = 1;
const SAVE_TEXT: usize = 2;
const RESET: usize = 3;

struct KeyCharCallback;

impl InputCallback for KeyCharCallback {
    fn add_char(&mut self, c: u32) {
        println!("add_char {}", c);
    }
}

fn main() {
    if args().len() > 2 {
        // Doesn't work if the windows_subsystem is enabled since terminal is not invoked
        panic!("Usage: percolationvisualizer <grid side length in integer (default: 5)>");
    }
    let n = args()
        .nth(1)
        .unwrap_or_else(|| String::from("5"))
        .parse::<usize>()
        .unwrap();

    let mut text_buf = String::with_capacity(AREA * 8);

    writeln!(&mut text_buf, "{}", n).unwrap();
    let mut buffer = vec![0; AREA];

    let mut window = Window::new(
        "Interactive Percolation Visualizer - ESC to exit",
        LENGTH,
        LENGTH + STATUS_HEIGHT,
        WindowOptions {
            ..WindowOptions::default()
        },
    )
    .expect("Unable to Open Window");

    window.set_input_callback(Box::new(KeyCharCallback {}));

    let mut menu = Menu::new("File").unwrap();

    menu.add_item("Save screenshot", SAVE_SCREEN)
        .shortcut(Key::S, MENU_KEY_CTRL)
        .build();
    menu.add_item("Save as text", SAVE_TEXT)
        .shortcut(Key::T, MENU_KEY_CTRL)
        .build();
    menu.add_item("Reset", RESET)
        .shortcut(Key::R, MENU_KEY_CTRL)
        .build();

    if let Some(menus) = window.get_posix_menus() {
        println!("Menus {:?}", menus);
    }
    window.add_menu(&menu);
    #[cfg(target_os = "windows")]
    window.set_icon(<Icon as std::str::FromStr>::from_str("percolation.ico").unwrap());
    let mut perc = Percolation::new(n);
    let block_half_length = half_length(n);
    let mut leftclick = false;
    let font = set_font();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some((idx, idy)) = window.get_mouse_pos(MouseMode::Discard) {
            let (idx, idy) = (
                ((idx / LENGTH as f32 * n as f32) + 1.) as usize,
                ((idy / LENGTH as f32 * n as f32) + 1.) as usize,
            );
            if leftclick {
                leftclick = false;
            } else {
                leftclick = window.get_mouse_down(MouseButton::Left);
            }
            if leftclick {
                if idy <= n {
                    perc.open(idy, idx);
                    writeln!(&mut text_buf, "{} {}", idy, idx).unwrap();
                    for idx in 1..=n {
                        for idy in 1..=n {
                            if perc.is_open(idy, idx) {
                                if perc.is_full(idy, idx) {
                                    fill_rect(idx, idy, n, block_half_length, 6801139, &mut buffer);
                                //rgb(103,198,243)
                                } else {
                                    fill_rect(
                                        idx,
                                        idy,
                                        n,
                                        block_half_length,
                                        16777215,
                                        &mut buffer,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut offscreenbuffer = buffer.clone();
        offscreenbuffer.extend(draw_status_bar(&perc, true, &font));
        if let Some(menu_id) = window.is_menu_pressed() {
            match menu_id {
                SAVE_SCREEN => {
                    save_screen(&mut offscreenbuffer);
                }
                SAVE_TEXT => {
                    save_text(&text_buf);
                }
                RESET => {
                    buffer = vec![0; AREA];
                    text_buf = String::with_capacity(AREA * 8);
                    perc = Percolation::new(n);
                }
                _ => {}
            }
        }

        // We unwrap here as we want this code to exit if it fails
        window
            .update_with_buffer(&offscreenbuffer, LENGTH, LENGTH + STATUS_HEIGHT)
            .unwrap();
    }
}
