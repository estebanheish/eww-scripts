use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

#[derive(Serialize)]
struct Hyprland {
    #[serde(rename = "workspaces", serialize_with = "serialize_ws")]
    ws: [Workspace; 15],
    #[serde(skip)]
    focused_ws: usize,
    #[serde(skip)]
    last_focused_ws: usize,
    screencast: bool,
    special_alive: bool,
    special_focused: bool,
}

fn serialize_ws<S>(ws: &[Workspace; 15], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ws: Vec<_> = ws.iter().filter(|x| x.alive).collect();
    let mut seq = serializer.serialize_seq(Some(ws.len()))?;
    for e in ws {
        seq.serialize_element(e)?;
    }
    seq.end()
}

#[derive(Serialize, Copy, Clone)]
struct Workspace {
    id: i32,
    focused: bool,
    alive: bool,
}

fn main() {
    let hyprland_instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    let socket =
        UnixStream::connect(format!("/tmp/hypr/{hyprland_instance}/.socket2.sock")).unwrap();
    let mut socket = BufReader::new(socket).lines();

    let mut hyprland = Hyprland::new();

    while let Some(Ok(line)) = socket.next() {
        let (event, arg) = line.split_once(">>").unwrap();
        hyprland.update(event, arg);
        hyprland.spit();
    }
}

impl Hyprland {
    fn new() -> Self {
        let mut ws = [Workspace {
            id: 0,
            focused: false,
            alive: false,
        }; 15];
        (0..15).for_each(|i| ws[i].id = i as i32);
        Self {
            ws,
            focused_ws: 0,
            last_focused_ws: 0,
            screencast: false,
            special_alive: false,
            special_focused: false,
        }
    }

    fn spit(&self) {
        println!("{}", serde_json::to_string(self).unwrap());
    }

    fn focus(&mut self, w: usize) {
        self.ws[self.focused_ws].focused = false;
        self.last_focused_ws = self.focused_ws;
        self.focused_ws = w;
        self.ws[w].focused = true;
        self.ws[w].alive = true;
    }

    fn update(&mut self, event: &str, arg: &str) {
        match event {
            "workspace" => {
                if let Ok(w) = arg.parse::<usize>() {
                    self.focus(w);
                }
            }
            "createworkspace" => {
                if arg == "special" {
                    self.special_alive = true;
                } else if let Ok(w) = arg.parse::<usize>() {
                    self.ws[w].alive = true;
                }
            }
            "destroyworkspace" => {
                if arg == "special" {
                    self.special_alive = false;
                } else if let Ok(w) = arg.parse::<usize>() {
                    self.ws[w].alive = false;
                }
            }
            "focusedmon" => {
                let (_, f) = arg.split_once(",").unwrap();
                if let Ok(w) = f.parse::<usize>() {
                    self.focus(w);
                }
            }
            "screencast" => match arg {
                "1,0" => self.screencast = true,
                "0,0" => self.screencast = false,
                _ => (),
            },
            "activespecial" => {
                self.special_focused = true;
                self.special_alive = true;
            }
            "activewindow" => {
                if self.special_focused {
                    self.special_focused = false;
                }
            }
            _ => (),
        }
    }
}
