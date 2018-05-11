//! A rust implementation of i3_vim_focus
//!
//! Original version from
//! https://faq.i3wm.org/question/3042/change-the-focus-of-windows-within-vim-and-i3-with-the-same-keystroke/
//!
//! Usage:
//!    i3-vim-focus [left|right|up|down]
//!
//! Requires that libxdo is installed

extern crate jwilm_xdo as xdo;
extern crate i3ipc;

#[macro_use]
extern crate log;

use std::env;
use std::str::FromStr;
use std::error::Error;
use std::fmt;


enum Direction {
    Up, Left, Down, Right
}

impl Direction {
    pub fn to_vim_direction(&self) -> &'static str {
        match *self {
            Direction::Up => "k",
            Direction::Down => "j",
            Direction::Left => "h",
            Direction::Right => "l",
        }
    }
}

impl FromStr for Direction {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "left" => Direction::Left,
            "right" => Direction::Right,
            "up" => Direction::Up,
            "down" => Direction::Down,
            _ => return Err("must specify one of left, right, up, or down"),
        })
    }
}

#[derive(Debug)]
enum FocusError {
    XdoError(xdo::Error),
    I3ipcConnectionError(i3ipc::EstablishError),
    I3ipcMessageError(i3ipc::MessageError),
}

impl fmt::Display for FocusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FocusError::XdoError(ref err) => err.fmt(f),
            FocusError::I3ipcMessageError(ref err) => err.fmt(f),
            FocusError::I3ipcConnectionError(ref err) => err.fmt(f),
        }
    }
}

impl Error for FocusError {
    fn description(&self) -> &str {
        match *self {
            FocusError::XdoError(ref err) => err.description(),
            FocusError::I3ipcConnectionError(ref err) => err.description(),
            FocusError::I3ipcMessageError(ref err) => err.description(),
        }
    }
}

impl From<xdo::Error> for FocusError {
    fn from(err: xdo::Error) -> FocusError {
        FocusError::XdoError(err)
    }
}

impl From<i3ipc::EstablishError> for FocusError {
    fn from(err: i3ipc::EstablishError) -> FocusError {
        FocusError::I3ipcConnectionError(err)
    }
}

impl From<i3ipc::MessageError> for FocusError {
    fn from(err: i3ipc::MessageError) -> FocusError {
        FocusError::I3ipcMessageError(err)
    }
}

fn xdo_i3_calls(name: &str, direction: Direction) -> Result<String, FocusError> {
    let xdo = xdo::Xdo::new()?;

    match xdo.get_active_window() {
        Ok(window) => {
            info!("Window = {:?}", window);

            match window.get_name() {
                Ok(window_name) => {
                    info!("Window name = {:?}", window_name);

                    if window_name.to_lowercase().contains("vim") {
                        let sequence = format!("Escape+g+w+{}", direction.to_vim_direction());
                        let mods = xdo.get_active_modifiers()?;
                        window.clear_active_modifiers(&mods)?;
                        window.send_keysequence(&sequence, None)?;
                        window.set_active_modifiers(&mods)?;

                        return Ok(format!("Vim sequence {:?} sent", sequence))
                    }
                },
                Err(e) => {
                    info!("Error getting window name: {:?}", e);
                }
            };
        },
        Err(e) => {
            info!("Error getting window: {:?}", e);
        }
    };

    let mut conn = i3ipc::I3Connection::connect()?;
    let command = format!("focus {}", name);
    let result = conn.command(&command)?;
    info!("command sent = {}", command);
    info!("result = {:?}", result);
    Ok(format!("Result = {:?}", result))
}

fn main() {

    let name = env::args().nth(1)
        .expect("direction was specified")
        .to_ascii_lowercase();

    info!("Direction name: {}", name);

    let direction = match Direction::from_str(&name) {
        Err(_) => { error!("No valid direction passed in arguments"); return; },
        Ok(d) => d
    };

    let result = xdo_i3_calls(&name[..], direction);
    match result {
        Err(e) => { error!("Error Running i3-vim-focus: {:?}", e); },
        Ok(s) => { info!("i3-vim-focus successfully ran: {:?}", s); },
    }
}
