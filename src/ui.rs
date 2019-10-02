use std::mem::MaybeUninit;
use std::io::Read;

use libc;

const FRAME_LENGTH: libc::c_long = 17_000_000;
const ONE_SECOND: libc::c_long = 1_000_000_000;

#[derive(Copy, Clone, PartialEq)]
pub enum Button {
    Up, Down, Left, Right, Rotate,
}


pub struct UI {
    old_settings: libc::termios,
    next_frame: libc::timespec,
    buttons: Vec<(u8, Button)>,
}

impl UI {
    pub fn init() -> Self {
        let mut old_settings = MaybeUninit::uninit();
        let old_settings = unsafe {
            libc::tcgetattr(
                libc::STDIN_FILENO,
                old_settings.as_mut_ptr());
            old_settings.assume_init()
        };
        let mut settings = old_settings.clone();
        unsafe { libc::cfmakeraw(&mut settings); }
        settings.c_cc[libc::VMIN] = 0;
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &settings);
        }
        /* clear screen, hide cursor */
        print!("\x1b[2J\x1b[?25l");

        let mut next_frame = MaybeUninit::uninit();

        let mut next_frame = unsafe {
            libc::clock_gettime(
                libc::CLOCK_MONOTONIC,
                next_frame.as_mut_ptr());
            next_frame.assume_init()
        };
        next_frame.tv_nsec += FRAME_LENGTH;

        return UI {
            old_settings, next_frame, buttons: Vec::new(),
        }
    }

    pub fn buttons(&self) -> &[(u8, Button)] {
        self.buttons.as_slice()
    }

    pub fn quit(&self){
        unsafe {
            libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSADRAIN,
                &self.old_settings);
        }
        print!("\x1b[0m\x1b[2J\x1b[1;1H\x1b[?25h");
    }

    pub fn frame(&mut self) -> bool {
        loop {
            let interrupted = unsafe {
                libc::clock_nanosleep(
                    libc::CLOCK_MONOTONIC,
                    libc::TIMER_ABSTIME,
                    &self.next_frame,
                    std::ptr::null_mut(),
                )
            };
            if interrupted == 0 {
                break;
            }
        }
        self.next_frame.tv_nsec += FRAME_LENGTH;
        while self.next_frame.tv_nsec >= ONE_SECOND {
            self.next_frame.tv_sec += 1;
            self.next_frame.tv_nsec -= ONE_SECOND;
        }
        
        let mut esc = false;
        let mut escseq = false;

        self.buttons.clear();
        loop {
            let ch = std::io::stdin().bytes().next();
            let ch = match ch {
                Some(Ok(c)) => c,
                _ => break,
            };
            let ch = match std::char::from_u32(ch.into()) {
                Some(c) => c,
                None => continue,
            };
            if esc && ch == '[' {
                esc = false;
                escseq = true;
                continue;
            } else if esc {
                /* esc key pressed by itself */
                return false;
            }
            let button_pressed;
            if escseq {
                escseq = false;
                button_pressed = match ch {
                    'A' => (2, Button::Up),
                    'B' => (2, Button::Down),
                    'C' => (2, Button::Right),
                    'D' => (2, Button::Left),
                    _ => continue,
                }
            } else {
                button_pressed = match ch {
                    '\x1b' => {
                        esc = true;
                        escseq = false;
                        continue
                    }
                    'q' => {
                        return false;
                    }
                    ' ' => (1, Button::Rotate),
                    'w' => (1, Button::Up),
                    'a' => (1, Button::Left),
                    's' => (1, Button::Down),
                    'd' => (1, Button::Right),
                    '\r' | '\n' => (2, Button::Rotate),
                    _ => continue,
                }
            }
            if !self.buttons.contains(&button_pressed) {
                self.buttons.push(button_pressed);
            }
        }
        return !esc;
    }
}
