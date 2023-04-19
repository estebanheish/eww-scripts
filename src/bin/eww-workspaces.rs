use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

type Ws = [bool; 15];

fn main() {
    let hyprland_instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    let socket =
        UnixStream::connect(format!("/tmp/hypr/{hyprland_instance}/.socket2.sock")).unwrap();
    let mut socket = BufReader::new(socket).lines();

    let (mut workspaces, mut focused) = init();

    while let Some(Ok(line)) = socket.next() {
        let (event, arg) = line.split_once(">>").unwrap();
        match event {
            "workspace" => focused = arg.parse().unwrap(),
            "createworkspace" => workspaces[arg.parse::<usize>().unwrap()] = true,
            "destroyworkspace" => workspaces[arg.parse::<usize>().unwrap()] = false,
            "focusedmon" => {
                let (_, f) = arg.split_once(",").unwrap();
                focused = f.parse().unwrap();
            }
            _ => continue,
        }
        spit(workspaces, focused);
    }
}

fn init() -> (Ws, u32) {
    let mut workspaces: Ws = [false; 15];
    for _ in 0..30 {
        match (get_focused(), get_workspaces()) {
            (Some(focused), Some(vws)) => {
                vws.iter().for_each(|w| workspaces[*w] = true);
                return (workspaces, focused);
            }
            _ => sleep(Duration::from_millis(500)),
        }
    }
    panic!("can't get initial values");
}

fn spit(ws: Ws, f: u32) {
    let mut out = String::new();
    out.push('[');
    for (i, w) in ws.iter().enumerate() {
        if *w {
            out.push_str(&format!(
                r#"{{ "id": {}, "focused": {} }}"#,
                i,
                i as u32 == f
            ));
            out.push(',');
        }
    }
    out.pop();
    out.push(']');
    println!("{}", out);
}

fn get_workspaces() -> Option<Vec<usize>> {
    Some(
        String::from_utf8(
            Command::new("hyprctl")
                .arg("workspaces")
                .output()
                .ok()?
                .stdout,
        )
        .ok()?
        .split("ID ")
        .skip(1)
        .map(|t| t.split(" ").next().unwrap().parse().unwrap())
        .collect(),
    )
}

fn get_focused() -> Option<u32> {
    let out = String::from_utf8(
        Command::new("hyprctl")
            .arg("activewindow")
            .output()
            .ok()?
            .stdout,
    )
    .ok()?;

    if let Some((_, s)) = out.split_once("workspace: ") {
        return Some(s.split(" ").next()?.parse().ok()?);
    }
    None
}
