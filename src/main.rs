use std::thread;
use std::time::Instant;
use std::time::Duration;
use minifb::WindowOptions;
use minifb::Window;
mod cpu;

const ROM_PATH: &str = "roms/";
const CPU_FREQ:u64 = 720; //instruction per second

const KEYS : [minifb::Key;16] = [ //index to key (ex: value 0 which is index 0 corresponds to X key which in reference to keyboard layout to chip8)
    minifb::Key::X,minifb::Key::Key1,minifb::Key::Key2,minifb::Key::Key3,    //0 1 2 3
    minifb::Key::Q,minifb::Key::W,minifb::Key::E,minifb::Key::A,             //4 5 6 7
    minifb::Key::S,minifb::Key::D,minifb::Key::Z,minifb::Key::C,             //8 9 A B
    minifb::Key::Key4,minifb::Key::R,minifb::Key::F,minifb::Key::V           //C D E F
];
fn main() {
    let mut chip8 = cpu::Chip8::new();
    let mut timer :Instant = Instant::now();
    let duration = Duration::from_micros(1_000_000 / CPU_FREQ);
    let timer_duration = Duration::from_micros(1_000_000 / 60);
    
    // instantiate audio player
    let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
    let source = rodio::source::SineWave::new(700.0);
    let player = rodio::Player::connect_new(handle.mixer());
    player.append(source);
    player.set_volume(0.2);
    player.pause();
    
    let Some(path) = std::env::args().nth(1) else {
        println!("Rom yok");
        return;
    };
    chip8.read_rom(&format!("{}{}",ROM_PATH,path));

    let gui_window_options = WindowOptions {
        scale : minifb::Scale::X16,
        scale_mode : minifb::ScaleMode::AspectRatioStretch,
        ..Default::default()
    };
    let mut gui_window = match Window::new("gui",64,32,gui_window_options) {
        Ok(t) => t,
        Err(err) => {
            println!("{}",err);
            return;
        }
    };

    while gui_window.is_open(){
        let clock = Instant::now();
        let inst = chip8.fetch();
        chip8.decode(inst);
        
        if timer.elapsed() > timer_duration { //60HZ
            if chip8.new_tick() { player.play();} else{ player.pause();}
            timer = Instant::now();
            gui_window.update_with_buffer(&chip8.render(), 64, 32).unwrap();
            for (index,key) in KEYS.into_iter().enumerate() {
                chip8.key_control((gui_window.is_key_down(key),index));
            }
        }
        let clock_now = clock.elapsed();
        if clock_now < duration { //720HZ
            thread::sleep(duration-clock_now);
        }
    }
}

