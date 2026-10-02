mod instructions;
                                //chip8 defaults
const VF_RESET : bool = true; //T
const MEMORY : bool = true; //T
const DISPWAIT : bool = true; //T
const CLIPPING : bool = true; //T
const SHIFTING : bool = false; //F
const JUMPING : bool = false; //F


const FONT: [u8;80] = [ //sprites of 0-F
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

const FONT_START : usize = 0;

pub struct Chip8 {
    v: [u8;16], //general purpose
    i :u16, //12 bit AR
    
    dt : u8, //delay timer
    st : u8,//sound timer
    
    pc : u16,
    sp : u8, //(next empty place on stack) pointer

    memory : [u8;4096],
    stack : [u16;16],
    
    screen : [[bool;64];32], //64x32 (64 width)(32 lenght)

    keyboard : [bool;16],

    can_write : bool,
}

impl Chip8 {
    pub fn new() -> Self{
        let mut s = Self {
            v : [0;16],
            i : 0,
            
            dt : 0, //default disabled
            st : 0,
            
            pc : 0x200,
            sp : 0x0, //stack writes upwards / reads downward

            memory : [0;4096],
            stack : [0;16],
            
            screen : [[false; 64];32],

            keyboard : [false;16],

            can_write : true,
        };

        s.memory[FONT_START.. FONT_START + FONT.len()].copy_from_slice(&FONT);
        s
    }

    pub fn read_rom(&mut self, path : &str) {
        let read_reslt =  std::fs::read(path);

        let read_vec = match read_reslt {
            Ok(t) => t,
            Err(e) => panic!("{e}"),
        };

        let starting_point : usize = self.pc as usize;

        if read_vec.len() > self.memory.len() - starting_point {
            panic!("Rom doesn't fit");
        }

        let end_point = starting_point + read_vec.len() ;

        self.memory[starting_point..end_point].copy_from_slice(&read_vec);

    }

    pub fn render (&self) -> [u32;64*32] { // true false layout to gui render format buffer
        let mut buffer : [u32; 64*32] = [0;64*32]; 
        for (k,row) in self.screen.into_iter().enumerate() {
            for (i,col) in row.into_iter().enumerate() {
                if col{ //white
                    buffer[k*64+i] = 0xFFFFFF;
                } else { //black
                    buffer[k*64+i] = 0;
                }
            }
        }
        buffer
    }

    pub fn key_control (&mut self, key : (bool,usize)) {
        self.keyboard[key.1] = key.0;
    }

    pub fn new_tick (&mut self) -> bool{
        self.dt = self.dt.saturating_sub(1); //to ensure minimum is 0
        self.st = self.st.saturating_sub(1);
        self.can_write = true;
        self.st > 0
    }

    pub fn fetch(&mut self) -> u16 {
        let inst = self.memory[self.pc as usize];
        self.pc = self.pc + 1;
        let inst2 = self.memory[self.pc as usize];
        self.pc = self.pc + 1;

        ((inst as u16) << 8) | (inst2 as u16) //instructions are stored as two rows
    }

    pub fn decode(&mut self, inst: u16) {
        //unpack the inst
        let nibbles : (u8,u8,u8,u8) = ((inst >> 12) as u8 , ((inst >> 8) & 0x0F) as u8, ((inst >> 4) & 0x0F) as u8, (inst & 0x0F) as u8);
                                                                        // & with 0x0F to ensure we only get the LSB nibble
        match nibbles {
            (0,0,0xE,0xE) => { //RET
                self.ret();
            }
            (0,0,0xE,0) => { //CLS
                self.cls();
            }
            (0,n1,n2,n3) => {  //SYS addr
                self.sys_addr((n1,n2,n3));
            }
            (1,n1,n2,n3) => { //JP addr
                self.jp_addr((n1,n2,n3));
            }
            (2,n1,n2,n3) => { //CALL addr
                self.call_addr((n1,n2,n3));
            }
            (3,x,k1,k2) => { //SE Vx, byte
                self.se_vx_byte((x,k1,k2));
            }
            (4,x,k1,k2) => { //SNE Vx, byte
                self.sne_vx_byte((x,k1,k2));
            }
            (5,x,y,0) => { //SE Vx, Vy
                self.se_vx_vy((x,y));
            }
            (6,x,k1,k2) => { //LD Vx, byte
                self.ld_vx_byte((x,k1,k2));
            }
            (7,x,k1,k2) => { //ADD Vx, byte
                self.add_vx_byte((x,k1,k2));
            }
            (8,x,y,0) => { //LD Vx, Vy
                self.ld_vx_vy((x,y));
            }
            (8,x,y,1) => { //OR Vx, Vy
                self.or_vx_vy((x,y));
            }
            (8,x,y,2) => { //AND Vx, Vy
                self.and_vx_vy((x,y));
            }
            (8,x,y,3) => { //XOR Vx, Vy
                self.xor_vx_vy((x,y));
            }
            (8,x,y,4) => { //ADD Vx, Vy
                self.add_vx_vy((x,y));
            }
            (8,x,y,5) => { //SUB Vx, Vy
                self.sub_vx_vy((x,y));
            }
            (8,x,y,6) => { //SHR Vx {, Vy}
                self.shr_vx_vy((x,y));
            }
            (8,x,y,7) => { //SUBN Vx, Vy
                self.subn_vx_vy((x,y));
            }
            (8,x,y,0xE) => { //SHL Vx {, Vy}
                self.shl_vx_vy((x,y));
            }
            (9,x,y,0) => { //SNE Vx, Vy
                self.sne_vx_vy((x,y));
            }
            (0xA,n1,n2,n3) => { //LD I, addr
                self.ld_i_addr((n1,n2,n3));
            }
            (0xB,n1,n2,n3) => { //JP V0, addr
                self.jp_v0_addr((n1,n2,n3));
            }
            (0xC,x,k1,k2) => { //RND Vx, byte
                self.rnd_vx_byte((x,k1,k2));
            }
            (0xD,x,y,n) => { //DRW Vx, Vy, nibble
                self.drw_vx_vy_nib((x,y,n));
            }
            (0xE,x,9,0xE) => { //SKP Vx
                self.skp_vx(x);
            }
            (0xE,x,0xA,1) => { //SKNP Vx
                self.sknp_vx(x);
            }
            (0xF,x,0,7) => { //LD Vx, DT
                self.ld_vx_dt(x);
            }
            (0xF,x,0,0xA) => { //LD Vx, K
                self.ld_vx_k(x);
            }
            (0xF,x,1,5) => { //LD DT, Vx
                self.ld_dt_vx(x);
            }
            (0xF,x,1,8) => { //LD ST, Vx
                self.ld_st_vx(x);
            }
            (0xF,x,1,0xE) => { //ADD I, Vx
                self.add_i_vx(x);
            }
            (0xF,x,2,9) => { //LD F, Vx
                self.ld_f_vx(x);
            }
            (0xF,x,3,3) => { //LD B, Vx
                self.ld_b_vx(x);
            }
            (0xF,x,5,5) => { //LD [I], Vx
                self.ld_i_vx(x);
            }
            (0xF,x,6,5) => { //LD Vx, [I]
                self.ld_vx_i(x);
            }
            (_,_,_,_) => { //unknown instruction
                println!("unknown instruction");
            }
        };
    }

}
