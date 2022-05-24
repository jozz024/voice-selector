#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

use arcropolis_api::*;
use percent_encoding::percent_decode_str;
use skyline_web::{ramhorns, Webpage};
use std::path::Path;
use std::marker::PhantomData;
use std::{collections::HashMap, sync::Mutex};
use bntx;
use std::io::{Read, Seek};
use std::io::SeekFrom;
use std::io::Cursor;
use binread::BinRead;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum Event {
    ArcFilesystemMounted,
    ModFilesystemMounted,
}

pub type EventCallbackFn = extern "C" fn(Event);

extern "C" {
    fn arcrop_register_event_callback(ty: Event, callback: EventCallbackFn);
}

static HTML_TEXT: &str = include_str!("index.html");
static CSS_TEXT: &str = include_str!("style.css");
static CHECK_IMG: &[u8] = include_bytes!("check.svg");

const VOICE_PATH: &str = "sound/bank/fighter_voice/";
const LOCALHOST: &str = "http://localhost/";
const FIGHTERS: &'static [&'static str] = &[
    "mario",
    "donkey",
    "link",
    "samus",
    "samusd",
    "yoshi",
    "kirby",
    "fox",
    "pikachu",
    "luigi",
    "ness",
    "captain",
    "purin",
    "peach",
    "daisy",
    "koopa",
    "ice_climber",
    "sheik",
    "zelda",
    "mariod",
    "pichu",
    "falco",
    "marth",
    "lucina",
    "younglink",
    "ganon",
    "mewtwo",
    "roy",
    "chrom",
    "gamewatch",
    "metaknight",
    "pit",
    "pitb",
    "szerosuit",
    "wario",
    "snake",
    "ike",
    "pzenigame",
    "pfushigisou",
    "plizardon",
    "diddy",
    "lucas",
    "sonic",
    "dedede",
    "pikmin",
    "lucario",
    "robot",
    "toonlink",
    "wolf",
    "murabito",
    "rockman",
    "wiifit",
    "rosetta",
    "littlemac",
    "gekkouga",
    "miifighter",
    "miiswordsman",
    "miigunner",
    "palutena",
    "pacman",
    "reflet",
    "shulk",
    "koopajr",
    "duckhunt",
    "ryu",
    "ken",
    "cloud",
    "kamui",
    "bayonetta",
    "inkling",
    "ridley",
    "simon",
    "richter",
    "krool",
    "shizue",
    "gaogaen",
    "packun",
    "jack",
    "brave",
    "buddy",
    "dolly",
    "master",
    "tantan",
    "pickel",
    "edge",
];

extern "C" {
    #[link_name = "\u{1}_ZN2nn3hid12GetNpadStateEPNS0_16NpadFullKeyStateERKj"]
    pub fn get_pro_state(arg1: u64, arg2: &u32);
    #[link_name = "\u{1}_ZN2nn3hid12GetNpadStateEPNS0_17NpadHandheldStateERKj"]
    pub fn get_handheld_state(arg1: u64, arg2: &u32);
    #[link_name = "\u{1}_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyDualStateERKj"]
    pub fn get_dual_joycon_state(arg1: u64, arg2: &u32);
    #[link_name = "\u{1}_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyLeftStateERKj"]
    pub fn get_left_joycon_state(arg1: u64, arg2: &u32);
    #[link_name = "\u{1}_ZN2nn3hid12GetNpadStateEPNS0_17NpadJoyRightStateERKj"]
    pub fn get_right_joycon_state(arg1: u64, arg2: &u32);
}

struct NpadHandheldState {
    update_count: i64,
    buttons: u64,
    l_stick_x: i32,
    l_stick_y: i32,
    r_stick_x: i32,
    r_stick_y: i32,
    flags: u32,
}

#[derive(std::cmp::PartialEq, std::clone::Clone, Copy)]
enum VoiceRegion {
    Eng,
    Jp,
    Default,
}

struct FileInfo {
    file_name: String,
    lang: VoiceRegion,
    code_name: String,
}

lazy_static! {
    static ref VOICES: Mutex<HashMap<u64, FileInfo>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref CHARA_VC_MAP: Mutex<HashMap<String, VoiceRegion>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

#[arc_callback]
fn arc_file_callback(hash: u64, data: &mut [u8]) -> Option<usize> {
    let path_;
    let lang = VOICES.lock().unwrap().get(&hash).unwrap().lang;
    let arc_path = VOICES.lock().unwrap().get(&hash).unwrap().file_name.clone();

    if lang == VoiceRegion::Eng {
        let region = "us_en";
        let region_nus3audio = format!("+{}.nus3audio", region);
        path_ = format!(
            "arc:/{}",
            arc_path.replace(".nus3audio", &region_nus3audio)
        );
    } else if lang == VoiceRegion::Jp {
        let region = "ja_jp";
        let region_nus3audio = format!("+{}.nus3audio", region);
        path_ = format!(
            "arc:/{}",
            arc_path.replace(".nus3audio", &region_nus3audio)
        );
    } else {
        let region = "us_en";
        let region_nus3audio = format!("+{}.nus3audio", region);
        path_ = format!(
            "arc:/{}",
            arc_path.replace(".nus3audio", &region_nus3audio)
        );
    }

    println!("{}", path_);

    match std::fs::read(path_) {
        Ok(file) => {
            data[..file.len()].copy_from_slice(&file);

            Some(file.len())
        }
        Err(_err) => None,
    }
}
fn write_all_pngs() {
    let stock_icon_path = "arc:/ui/replace/chara/chara_2";

    for character in FIGHTERS {

        let realpath = format!("{}/chara_2_{}_00.bntx", stock_icon_path, character);
        let png_path = "sd:/atmosphere/contents/01006a800016e000/manual_html/html-document/contents.htdocs/img/bntx/";

        println!("{}", realpath);

        match std::fs::read(realpath) {
            Ok(file) => {

                if !Path::new(&png_path).is_dir() {
                    std::fs::create_dir_all(&png_path);
                }

                let mut cursor = Cursor::new(file);

                let mut bntx_file = bntx::BntxFile::read(&mut cursor).unwrap();

                let to_write = format!("sd:/atmosphere/contents/01006a800016e000/manual_html/html-document/contents.htdocs/img/bntx/chara_2_{}_00.png", character);

                let img = bntx_file.to_image();
                img.save(to_write);
            }
            Err(_err) => (),
        };
    }
}

#[derive(ramhorns::Content)]
struct Info {
    chara_names: Vec<CharaName>,
}

#[derive(ramhorns::Content)]
struct CharaName(String, String);

pub fn show_menu() {
    let tpl = ramhorns::Template::new(HTML_TEXT).unwrap();

    let render = tpl.render(&Info {
        chara_names: {
            let mut cv: Vec<CharaName> = Vec::new();
            for x in FIGHTERS {
                cv.push(CharaName(x.to_string(), {
                    let reg = CHARA_VC_MAP
                        .lock()
                        .unwrap()
                        .get(&x.to_string())
                        .unwrap()
                        .clone();
                    if reg == VoiceRegion::Eng {
                        "eng".to_string()
                    } else if reg == VoiceRegion::Jp {
                        "jp".to_string()
                    } else {
                        "default".to_string()
                    }
                }));
            }
            cv
        },
    });

    let resp = Webpage::new()
        .htdocs_dir("contents")
        .file("index.html", &render)
        .file("style.css", CSS_TEXT)
        .file("check.svg", CHECK_IMG)
        .background(skyline_web::Background::BlurredScreenshot)
        .boot_display(skyline_web::BootDisplay::Default)
        .open()
        .unwrap();

    match resp.get_last_url() {
        Ok(r) => {
            match r {
                LOCALHOST => {}
                url => {
                    let res = percent_decode_str(&url[LOCALHOST.len()..])
                        .decode_utf8_lossy()
                        .into_owned();
                    let split_res = res.split("CSK_SPLIT").collect::<Vec<&str>>();
        
                    let split_res = &split_res[..split_res.len() - 1];
        
                    for s in split_res {
                        let info: Vec<&str> = s.split("=").collect::<Vec<&str>>();
                        let region: VoiceRegion;
        
                        match info[1] {
                            "eng" => region = VoiceRegion::Eng,
                            "jp" => region = VoiceRegion::Jp,
                            "default" => region = VoiceRegion::Default,
                            _ => {
                                println!("{} :bruhchu:", info[0]);
                                region = VoiceRegion::Eng;
                            }
                        }

                        if let Some(z) = CHARA_VC_MAP.lock().unwrap().get_mut(&info[0][..]) {
                            *z = region;
                        }
        
                        for x in 0..8 {
                            let file_hash =
                                hash40(&format!("{}vc_{}_c0{}.nus3audio", VOICE_PATH, info[0], x)).as_u64();
                            println!("{:#x}", file_hash);
                            if let Some(z) = VOICES.lock().unwrap().get_mut(&file_hash) {
                                z.lang = region;
                            }
                        }
                    }
                }
            }
        }
        Err(_err) => println!("{}", _err)

    }
}
pub extern "C" fn main_real(event: Event) {
    for x in FIGHTERS {
        CHARA_VC_MAP
            .lock()
            .unwrap()
            .insert(x.to_string(), VoiceRegion::Default);

        let vc_file = format!("vc_{}.nus3audio", x);

        for y in 0..8 {
            let callback_file = format!("vc_{}_c0{}.nus3audio", x, y);
            let path_ = format!("{}{}", VOICE_PATH, &callback_file);
            let hash = hash40(&path_);

            arc_file_callback::install(hash, 10000000);
            VOICES.lock().unwrap().insert(
                hash.as_u64(),
                FileInfo {
                    file_name: path_.to_string(),
                    lang: VoiceRegion::Default,
                    code_name: x.to_string(),
                },
            );
        }
    }

    write_all_pngs();

    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(10));
        let mut toggle_flag: bool = false;
        let mut state = NpadHandheldState {
            update_count: 0,
            buttons: 0,
            l_stick_x: 0,
            l_stick_y: 0,
            r_stick_x: 0,
            r_stick_y: 0,
            flags: 0,
        };
        let mut controller_value: u32 = 0x20;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            unsafe {
                get_handheld_state(
                    &mut state as *mut NpadHandheldState as u64,
                    &controller_value,
                );
                if (state.buttons & (1 << 9)) != 0 && (state.buttons & (1 << 8)) != 0 {
                    toggle_flag = true;
                }
                for x in 0..8 {
                    if toggle_flag {
                        break;
                    }
                    controller_value = x as u32;
                    get_pro_state(
                        &mut state as *mut NpadHandheldState as u64,
                        &controller_value,
                    );
                    if (state.buttons & (1 << 9)) != 0 && (state.buttons & (1 << 8)) != 0 {
                        toggle_flag = true;
                        break;
                    }
                }

                if toggle_flag {
                    show_menu();
                }
            }
            toggle_flag = false;
        }
    });
}
#[skyline::main(name = "voice-selector")]
pub fn main() {
    unsafe {
        arcrop_register_event_callback(Event::ArcFilesystemMounted, main_real);
    }
}
