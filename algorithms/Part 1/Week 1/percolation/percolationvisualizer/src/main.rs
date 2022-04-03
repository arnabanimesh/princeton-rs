#![windows_subsystem = "windows"]

use minifb::{Icon, InputCallback, Key, Menu, Window, WindowOptions, MENU_KEY_CTRL};
use percolation::{gui::*, Percolation};
use std::{thread, time::Duration};

const DELAY: u64 = 1000; // increase this value to slow the animation speed and vice versa

const OPEN_INPUT: usize = 1;
const SAVE_SCREEN: usize = 2;

struct KeyCharCallback;

impl InputCallback for KeyCharCallback {
    fn add_char(&mut self, c: u32) {
        println!("add_char {}", c);
    }
}

fn main() {
    let mut input_file;
    let mut input = [].iter();
    let mut n_deref = 0;
    let mut buffer = vec![0; AREA];

    let mut window = Window::new(
        "Percolation Visualizer - ESC to exit",
        LENGTH,
        LENGTH + STATUS_HEIGHT,
        WindowOptions {
            ..WindowOptions::default()
        },
    )
    .expect("Unable to Open Window");

    // Modify default speed dynamically with respect to the size of the grid
    window.limit_update_rate(Some(std::time::Duration::from_micros(DELAY)));

    window.set_input_callback(Box::new(KeyCharCallback {}));

    let mut menu = Menu::new("File").unwrap();
    menu.add_item("Open new input file", OPEN_INPUT)
        .shortcut(Key::O, MENU_KEY_CTRL)
        .build();

    menu.add_item("Save screenshot", SAVE_SCREEN)
        .shortcut(Key::S, MENU_KEY_CTRL)
        .build();

    if let Some(menus) = window.get_posix_menus() {
        println!("Menus {:?}", menus);
    }
    window.add_menu(&menu);

    #[cfg(target_os = "windows")]
    window.set_icon(<Icon as std::str::FromStr>::from_str("percolation.ico").unwrap());

    let mut perc = Percolation::default();
    let mut block_half_length = 0;
    let font = set_font();

    let mut init = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some(&idy) = input.next() {
            let &idx = input.next().unwrap();
            perc.open(idy, idx);
            for idx in 1..=n_deref {
                for idy in 1..=n_deref {
                    if perc.is_open(idy, idx) {
                        if perc.is_full(idy, idx) {
                            fill_rect(idx, idy, n_deref, block_half_length, 6801139, &mut buffer);
                        //rgb(103,198,243)
                        } else {
                            fill_rect(idx, idy, n_deref, block_half_length, 16777215, &mut buffer);
                        }
                    }
                }
            }
        } else {
            thread::sleep(Duration::from_millis(100));
        }
        let mut offscreenbuffer = buffer.clone();
        offscreenbuffer.extend(draw_status_bar(&perc, init, &font));
        if let Some(menu_id) = window.is_menu_pressed() {
            match menu_id {
                SAVE_SCREEN => {
                    save_screen(&mut offscreenbuffer);
                }
                OPEN_INPUT => {
                    input_file = open();
                    input = input_file.iter();
                    if let Some(n) = input.next() {
                        n_deref = *n;
                        buffer = vec![0; AREA];
                        perc = Percolation::new(n_deref);
                        block_half_length = half_length(n_deref);
                        init = true;
                    }
                }
                _ => (),
            }
        }

        // We unwrap here as we want this code to exit if it fails
        window
            .update_with_buffer(&offscreenbuffer, LENGTH, LENGTH + STATUS_HEIGHT)
            .unwrap();
    }
}
