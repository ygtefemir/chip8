use super::Chip8;
use super::{VF_RESET, MEMORY, DISPWAIT, CLIPPING, SHIFTING, JUMPING, FONT_START};

impl Chip8 {
    //00E0 clear display
    pub(super) fn cls(&mut self){
        self.screen = [[false;64];32];
    }
    //00EE return from subroutine
    pub(super) fn ret(&mut self){
        self.sp -=1;
        self.pc = self.stack[self.sp as usize];
    }
    //0nnn jmp to machine code
    pub(super) fn sys_addr(&mut self, _var : (u8,u8,u8)){
        //presumed to be not used
    }
    //1nnn jmp to addr
    pub(super) fn jp_addr(&mut self, addr : (u8,u8,u8)){
        let addr16: u16 = comb12(addr);
        self.pc = addr16;
    }
    //2nnn call subroutine addr
    pub(super) fn call_addr(&mut self, addr : (u8,u8,u8)){
        self.stack[self.sp as usize] = self.pc;
        self.sp +=1;
        self.pc = comb12(addr);
    }
    //3xkk if Vx==kk skip
    pub(super) fn se_vx_byte(&mut self, var : (u8,u8,u8)){
                                              //x k1 k2
        if self.v[var.0 as usize] == comb8((var.1,var.2)){
            self.pc+=2;
        }
    }
    //4xkk if Vx!=kk skip
    pub(super) fn sne_vx_byte(&mut self, var : (u8,u8,u8)){
                                                //x k1 k2
        if self.v[var.0 as usize] != comb8((var.1,var.2)){
            self.pc+=2;
        }
    }
    //5xy0 if Vx==Vy skip
    pub(super) fn se_vx_vy(&mut self, var: (u8,u8)){
                                            //x,y
        if self.v[var.0 as usize] == self.v[var.1 as usize]{
            self.pc+=2;
        }
    }
    //6xkk Vx=kk
    pub(super) fn ld_vx_byte(&mut self, var : (u8,u8,u8)){
                                              //x,k1,k2
        self.v[var.0 as usize] = comb8((var.1,var.2));
    }
    //7xkk Vx+=kk
    pub(super) fn add_vx_byte(&mut self, var : (u8,u8,u8)){
                                               //x,k1,k2
        self.v[var.0 as usize] = self.v[var.0 as usize].wrapping_add(comb8((var.1,var.2)));
    }
    //8xy0 Vx=Vy
    pub(super) fn ld_vx_vy (&mut self, var: (u8,u8)){
                                             //x,y
        self.v[var.0 as usize] = self.v[var.1 as usize];
    }
    //8xy1 Vx=Vx OR Vy
    pub(super) fn or_vx_vy (&mut self, var : (u8,u8)){
                                              //x,y
        self.v[var.0 as usize] = self.v[var.0 as usize] | self.v[var.1 as usize];
        if VF_RESET{ self.v[0xF] = 0;}
    }
    //8xy2 Vx= Vx AND Vy
    pub(super) fn and_vx_vy (&mut self, var : (u8,u8)){
                                               //x,y
        self.v[var.0 as usize] = self.v[var.0 as usize] & self.v[var.1 as usize];
        if VF_RESET{ self.v[0xF] = 0;}
    }
    //8xy3 Vx= Vx XOR Vy
    pub(super) fn xor_vx_vy (&mut self, var: (u8,u8)){
                                              //x,y
        self.v[var.0 as usize] = self.v[var.0 as usize] ^ self.v[var.1 as usize];
        if VF_RESET{ self.v[0xF] = 0;}
    }
    //8xy4 Vx+=Vy, VF=Carry
    pub(super) fn add_vx_vy(&mut self, var : (u8,u8)){
                                              //x,y
        let result = self.v[var.0 as usize].overflowing_add(self.v[var.1 as usize]); //(sum,carry(t/f))
        self.v[var.0 as usize] = result.0;
        self.v[0xF] = result.1 as u8;
    }
    //8xy5 Vx -=Vy Vf=NOT BORROW
    pub(super) fn sub_vx_vy(&mut self , var : (u8,u8)) {
                                              //x,y
        let result = self.v[var.0 as usize].overflowing_sub(self.v[var.1 as usize]);
        self.v[var.0 as usize] = result.0;
        self.v[0xF] = (!result.1 as u8); //bu işlemde borrow VARSA flag 0 oluyor
    }
    //8xy6 SHR Vx 1, VF = if LSB was 1 (shift side)
    pub(super) fn shr_vx_vy(&mut self, var : (u8,u8)){
                                             //x,y
        if !SHIFTING {self.v[var.0 as usize] = self.v[var.1 as usize];}
        let flag = self.v[var.0 as usize] & 1; //lsb check
        self.v[var.0 as usize] = self.v[var.0 as usize] >> 1;
        self.v[0xF] = flag; //flag should be assigned last, Vx might be VF the flag reg
    }
    //8xy7 Vx=Vy-Vx Vf=NOT borrow
    pub(super) fn subn_vx_vy(&mut self, var : (u8,u8)){
                                                //x,y
        let result = self.v[var.1 as usize].overflowing_sub(self.v[var.0 as usize]);
        self.v[var.0 as usize] = result.0;
        self.v[0xF] = (!result.1 as u8);
    }
    //8xyE Vx Shift left 1, VF = if MSB was 1 (shift side)
    pub(super) fn shl_vx_vy(&mut self, var : (u8,u8)){
                                              //x,y
        if !SHIFTING {self.v[var.0 as usize] = self.v[var.1 as usize];}
        let flag = (self.v[var.0 as usize] & 0x80) >> 7; //msb check
        self.v[var.0 as usize] = self.v[var.0 as usize] << 1;
        self.v[0xF] = flag; //flag should be assigned last, Vx might be VF the flag reg
    }
    //9xy0 if Vx!=Vy skip
    pub(super) fn sne_vx_vy(&mut self, var: (u8,u8)) {
        if self.v[var.0 as usize] != self.v[var.1 as usize]{
            self.pc+=2;
        }
    }
    //ANNN I =nnn
    pub(super) fn ld_i_addr(&mut self, nibbles : (u8,u8,u8) ) {
        self.i = comb12(nibbles);
    }
    //Bnnn jmp to V0 + addr
    pub(super) fn jp_v0_addr(&mut self, addr : (u8,u8,u8)){
        let sum : u16 = match JUMPING {
            true => self.v[addr.0 as usize] as u16 + comb12(addr),
            false => self.v[0] as u16 + comb12(addr),
        };
        self.pc = (sum) % self.memory.len() as u16; //index taşması olmasın
    }
    //Cxkk Vx = rnd & kk
    pub(super) fn rnd_vx_byte(&mut self, var : (u8,u8,u8)){
        let num : u8 = fastrand::u8(..);
        self.v[var.0 as usize] = num & comb8((var.1,var.2));
    }
    //DXYN I dan başlayarak n byte(ram satırı)lık sprite ı (Vx,Vy) noktasında yazdır, çarpışma olursa VF flag aç (XOR sonucu  1 1 -> 0 olursa)
    pub(super) fn drw_vx_vy_nib(&mut self, nibbles : (u8,u8,u8)){
                                                     //x y n
        //y ordinate x axis
        if !self.can_write { // for DISP.WAIT quirk
            self.pc-=2;
            return;
        }
        let mut j : u8 = 0;
        let x = self.v[nibbles.0 as usize] as usize % self.screen[0].len(); //matrix coll
        let mut y = self.v[nibbles.1 as usize] as usize % self.screen.len(); //matrix row
        self.v[0xF] = 0;
        while j < nibbles.2{ // for every byte
            let temp = self.memory[self.i as usize + j as usize]; //get the sprite row from ram
                for bit in (0..8) { // every bit on the sprite row corresponds to a pixel, so we will search for every bit on the byte
                    let mask = 0x80 >> bit; //starting from 1000 0000 shift to right and create the mask which we will use to get corresponding bit which we need
                    //need to use this mask since we cant do bit level operations
                    let sprite = temp & mask; // to be filled ?
                    
                    let mut paint :bool = false;
                    if(sprite != 0) {paint = true;}
                    
                    let mut offset_x = x+bit; //starting from x point shift to right to the corresponding point for bit
                    if offset_x >=  self.screen[y].len() && CLIPPING { //if clipping is enabled and we overflow the screen, rest of the sprite is dropped
                        continue;
                    }else{ //if not it should be overlapped to first index (left of the screen)
                        offset_x = offset_x % self.screen[y].len();
                    }
                    let pixel = self.screen[y][offset_x];
                    
                    let result = pixel ^ paint; //to determine if we are painting a already painted pixel (which would remove the paint)
                    if result == false && pixel == true { //did we remove a paint ?
                        self.v[0xF] = 1;
                    }
                    self.screen[y][offset_x] = result;
            }
            j+=1;//next sprite
            y +=1; //next row
            if y >= self.screen.len() && CLIPPING { //if clipping is enabled whilist reaching the end of the screen
                break;                              //rendering of the sprite is considered done
            }else{
                y = y%self.screen.len(); //if clipping is disabled just overlap to the top of the screen
            }
        }
        if DISPWAIT {self.can_write = false;}
    }
    //Ex9E skip insturction if Vx==key_pressed(key which is down rn)
    pub(super) fn skp_vx(&mut self, x : u8){
        if self.keyboard[self.v[x as usize] as usize]{
            self.pc+=2;
        }
    }
    //ExA1 skip insturction if Vx==key_not_pressed(key which is not down rn)
    pub(super) fn sknp_vx(&mut self, x : u8){
        if !self.keyboard[self.v[x as usize] as usize]{
            self.pc+=2;
        }    
    }
    //Fx07 vx = dt
    pub(super) fn ld_vx_dt(&mut self, x : u8){
        self.v[x as usize] = self.dt;
    }
    //Fx0A store the pressed down key on Vx
    pub(super) fn ld_vx_k(&mut self, x : u8){
        for i in(0..16){
            if self.keyboard[i]{
                self.v[x as usize] = i as u8; //first detected key in order of keyboard should be stored nothing else
                return;
            }
        }
        self.pc -=2; // instruction will work until a key is stored
    }
    //Fx15 dt = Vx
    pub(super) fn ld_dt_vx(&mut self, x : u8){
        self.dt = self.v[x as usize];
    }
    //Fx18 st = Vx  
    pub(super) fn ld_st_vx(&mut self, x : u8){
        self.st = self.v[x as usize];
    }
    //Fx1E I=I+Vx
    pub(super) fn add_i_vx(&mut self, x : u8){
        self.i += self.v[x as usize] as u16;
    }
    //Fx29 I set to digit of sprite corresponding to Vx (from the start of the ram FONTS)
    pub(super) fn ld_f_vx(&mut self, x : u8){
        self.i = (self.v[x as usize] as u16 * 5) + FONT_START as u16; //multipled by five since every sprite of digits takes up 5 rows of space on ram
    }
    //Fx33 store Vx BCD on I ==> store Vx decimal as, I -> hundreds, I+1 -> tens, I+2-> ones
    pub(super) fn ld_b_vx(&mut self, x : u8){
        let temp = self.v[x as usize];
        self.memory[self.i as usize] = temp /100;
        self.memory[self.i as usize + 1] = temp %100 /10;
        self.memory[self.i as usize + 2] = temp %10;
    }
    //Fx55 store register V0 through Vx in memory starting from I
    pub(super) fn ld_i_vx(&mut self, x : u8){
        for i in (0..(x+1) as usize){
            self.memory[self.i as usize + i] = self.v[i];
        }
        if MEMORY {self.i += (x+1) as u16;}
    }
    //Fx65 read registers V0 through Vx from memory starting from I
    pub(super) fn ld_vx_i(&mut self, x : u8){
        for i in (0..(x+1) as usize){
            self.v[i] = self.memory[self.i as usize+i];
        }
        if MEMORY {self.i += (x+1) as u16;}    
    }

}

//bit packing helper fns

fn comb8 (var: (u8,u8)) ->u8 {
    (var.0 << 4 | var.1)
}

fn comb12 (var: (u8,u8,u8)) -> u16 {
    ((var.0 as u16)<< 8) | ((var.1 as u16)<< 4) | (var.2 as u16)
}