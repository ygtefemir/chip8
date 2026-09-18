const INP: [u8;2] = [//2 placeholder
    0x01, 0x02,
]; //değerler sonra doldurulacak



struct Chip8 {
    v: [u8;16], //general purpose
    i :u16, //12 bit AR
    
    dt : u8, //delay timer
    st : u8,//sound timer
    
    pc : u16,
    sp : u8, //sonraki boş alan stackteki

    memory : [u8;4096],
    stack : [u16;16],
    
    screen : [[bool;64];32], //64x32 (64 width)(32 lenght)
}

impl Chip8 {
    fn new() -> Self{
        let mut s = Self {
            v : [0;16],
            i : 0, //şuanki değeri önemsiz komutlarda önemli
            
            dt : 0, //default deaktif
            st : 0,
            
            pc : 0x200,
            sp : 0x0, //stack yukarı doğru yazıyor

            memory : [0;4096],
            stack : [0;16],
            
            screen : [[false; 64];32],
        };

        s.memory[..INP.len()].copy_from_slice(&INP);//interpeter kısmını doldur

        s
    }

    fn read_rom(&mut self, path : &str) {
        let read_reslt =  std::fs::read(path); //Result döndü vec çek

        let read_vec = match read_reslt {
            Ok(t) => t,
            Err(e) => panic!("{e}"),
        };

        let starting_point : usize = self.pc as usize;

        if read_vec.len() > self.memory.len() - starting_point {
            panic!("Rom sığmıyor");
        }

        let end_point = starting_point + read_vec.len() ;

        self.memory[starting_point..end_point].copy_from_slice(&read_vec);

    }

    fn print_screen (&self) {
        let mut text : String = "".to_string();
        for row in self.screen {
            for col in row {
                if col{
                text.push('\u{2588}');
                } else {
                    text.push(' ');
                }
            }
            text.push_str("\n");
        }
        println!("{}",text);
    }

    fn fetch(&mut self) -> u16 {
        let inst = self.memory[self.pc as usize];
        self.pc = self.pc + 1;
        let inst2 = self.memory[self.pc as usize];
        self.pc = self.pc + 1;

        ((inst as u16) << 8) | (inst2 as u16) //inst 8 bit sola kaydırıldı
    }

    fn decode(&mut self, inst: u16) {
        let nibbles : (u8,u8,u8,u8) = ((inst >> 12) as u8 , ((inst >> 8) & 0x0F) as u8, ((inst >> 4) & 0x0F) as u8, (inst & 0x0F) as u8); //0x0F sayesinde sadece alt nibble değerlenin kalması sağlandı

        match nibbles {
            (0,0,0xE,0xE) => { //RET
                println!("RET");
                self.ret();
            }
            (0,0,0xE,0) => { //CLS
                println!("CLS");
                self.cls();
            }
            (0,n1,n2,n3) => {  //SYS addr
                println!("SYS addr");
                self.sys_addr((n1,n2,n3));
            }
            (1,n1,n2,n3) => { //JP addr
                println!("JP addr");
                self.jp_addr((n1,n2,n3));
            }
            (2,n1,n2,n3) => { //CALL addr
                println!("CALL addr");
                self.call_addr((n1,n2,n3));
            }
            (3,x,k1,k2) => { //SE Vx, byte
                println!("SE Vx, byte");
                self.se_vx_byte((x,k1,k2));
            }
            (4,_,_,_) => { //SNE Vx, byte
                println!("SNE Vx, byte");
            }
            (5,x,y,0) => { //SE Vx, Vy
                println!("SE Vx, Vy");
                self.se_vx_vy((x,y));
            }
            (6,x,k1,k2) => { //LD Vx, byte
                println!("LD Vx, byte");
                self.ld_vx_byte((x,k1,k2));
            }
            (7,x,k1,k2) => { //ADD Vx, byte
                println!("ADD Vx, byte");
                self.add_vx_byte((x,k1,k2));
            }
            (8,_,_,0) => { //LD Vx, Vy
                println!("LD Vx, Vy");
            }
            (8,_,_,1) => { //OR Vx, Vy
                println!("OR Vx, Vy");
            }
            (8,_,_,2) => { //AND Vx, Vy
                println!("AND Vx, Vy");
            }
            (8,_,_,3) => { //XOR Vx, Vy
                println!("XOR Vx, Vy");
            }
            (8,_,_,4) => { //ADD Vx, Vy
                println!("ADD Vx, Vy");
            }
            (8,_,_,5) => { //SUB Vx, Vy
                println!("SUB Vx, Vy");
            }
            (8,_,_,6) => { //SHR Vx {, Vy}
                println!("SHR Vx {{, Vy}}");
            }
            (8,_,_,7) => { //SUBN Vx, Vy
                println!("SUBN Vx, Vy");
            }
            (8,_,_,0xE) => { //SHL Vx {, Vy}
                println!("SHL Vx {{, Vy}}");
            }
            (9,x,y,0) => { //SNE Vx, Vy
                println!("SNE Vx, Vy");
                self.sne_vx_vy((x,y));
            }
            (0xA,n1,n2,n3) => { //LD I, addr
                println!("LD I, addr");
                self.ld_i_addr((n1,n2,n3));
            }
            (0xB,_,_,_) => { //JP V0, addr
                println!("JP V0, addr");
            }
            (0xC,_,_,_) => { //RND Vx, byte
                println!("RND Vx, byte");
            }
            (0xD,x,y,n) => { //DRW Vx, Vy, nibble
                println!("DRW Vx, Vy, nibble");
                self.drw_vx_vy_nib((x,y,n));
            }
            (0xE,_,9,0xE) => { //SKP Vx
                println!("SKP Vx");
            }
            (0xE,_,0xA,1) => { //SKNP Vx
                println!("SKNP Vx");
            }
            (0xF,_,0,7) => { //LD Vx, DT
                println!("LD Vx, DT");
            }
            (0xF,_,0,0xA) => { //LD Vx, K
                println!("LD Vx, K");
            }
            (0xF,_,1,5) => { //LD DT, Vx
                println!("LD DT, Vx");
            }
            (0xF,_,1,8) => { //LD ST, Vx
                println!("LD ST, Vx");
            }
            (0xF,_,1,0xE) => { //ADD I, Vx
                println!("ADD I, Vx");
            }
            (0xF,_,2,9) => { //LD F, Vx
                println!("LD F, Vx");
            }
            (0xF,_,3,3) => { //LD B, Vx
                println!("LD B, Vx");
            }
            (0xF,_,5,5) => { //LD [I], Vx
                println!("LD [I], Vx");
            }
            (0xF,_,6,5) => { //LD Vx, [I]
                println!("LD Vx, [I]");
            }
            (_,_,_,_) => { //unknown instruction
                println!("unknown instruction");
            }
        };
    }
    
    //INSTURCTIONS
    
    //00EE
    fn ret(&mut self){
        self.sp -=1;
        self.pc = self.stack[self.sp as usize];
    }
    //00E0
    fn cls(&mut self) {
        self.screen = [[false;64];32];
    }
    //0nnn
    fn sys_addr(&mut self, var : (u8,u8,u8)){
        //kullanacak mıyız? şimdilik boş
    }
    //1nnn
    fn jp_addr(&mut self, addr : (u8,u8,u8)){
        let addr16: u16 = comb12(addr);
        self.pc = addr16;
    }
    //2nnn
    fn call_addr(&mut self, addr : (u8,u8,u8)){
        self.stack[self.sp as usize] = self.pc;
        self.sp +=1;
        self.pc = comb12(addr);
    }
    //3xkk
    fn se_vx_byte(&mut self, var : (u8,u8,u8)){
        //x k1 k2
        if self.v[var.0 as usize] == comb8((var.1,var.2)){
            self.pc+=2;
        }
    }
    //4xkk
    fn sne_vx_byte(&mut self, var : (u8,u8,u8)){
        //x k1 k2
        if self.v[var.0 as usize] != comb8((var.1,var.2)){
            self.pc+=2;
        }
    }
    //5xy0
    fn se_vx_vy(&mut self, var: (u8,u8)){
        //x,y
        if self.v[var.0 as usize] == self.v[var.1 as usize]{
            self.pc+=2;
        }
    }
    //6xkk
    fn ld_vx_byte(&mut self, var : (u8,u8,u8)){
        //x,k1,k2
        self.v[var.0 as usize] = comb8((var.1,var.2));
    }
    //7xkk
    fn add_vx_byte(&mut self, var : (u8,u8,u8)){
        //x,k1,k2
        self.v[var.0 as usize] = self.v[var.0 as usize].wrapping_add(comb8((var.1,var.2)));
    }
    //8xy0
    fn ld_vx_vy (&mut self, var: (u8,u8)){
        //x,y
        self.v[var.0 as usize] = self.v[var.1 as usize];
    }
    //8xy1
    fn or_vx_vy (&mut self, var : (u8,u8)){
        //x,y
        self.v[var.0 as usize] = self.v[var.0 as usize] | self.v[var.1 as usize];
    }
    //8xy2
    fn and_vx_vy (&mut self, var : (u8,u8)){
        //x,y
        self.v[var.0 as usize] = self.v[var.0 as usize] & self.v[var.1 as usize];
    }
    //8xy3
    fn xor_vx_vy (&mut self, var: (u8,u8)){
        //x,y
        self.v[var.0 as usize] = self.v[var.0 as usize] ^ self.v[var.1 as usize];
    }
    //8xy4
    fn add_vx_vy(&mut self, var : (u8,u8)){
        //x,y
        let result = self.v[var.0 as usize].overflowing_add(self.v[var.1 as usize]); //(sum,carry(t/f))
        self.v[var.0 as usize] = result.0;
        self.v[0xF] = result.1 as u8;
    }
    //8xy5
    fn sub_vx_vy(&mut self , var : (u8,u8)) {
        //x,y
        let result = self.v[var.0 as usize].overflowing_sub(self.v[var.1 as usize]);
        self.v[var.0 as usize] = result.0;
        self.v[0xF] = (!result.1 as u8); //bu işlemde borrow VARSA flag 0 oluyor
    }
    //8xy6
    fn shr_vx_vy(&mut self, var : (u8,u8)){
        //x,y (y opsiyonel)
        let flag = self.v[var.0 as usize] & 1; //lsb check
        self.v[var.0 as usize] = self.v[var.0 as usize] >> 1;
        self.v[0xF] = flag; //flag atması en son olmalı belki x değeri 0xF olabilir
    }
    //8xy7
    fn subn_vx_vy(&mut self, var : (u8,u8)){
        //x,y
        let result = self.v[var.1 as usize].overflowing_sub(self.v[var.0 as usize]);
        self.v[var.0 as usize] = result.0;
        self.v[0xF] = (!result.1 as u8); //bu işlemde borrow VARSA flag 0 oluyor
    }
    //8xyE
    fn shl_vx_vy(&mut self, var : (u8,u8)){
        //x,y (y opsiyonel)
        let flag = (self.v[var.0 as usize] & 0x80) >> 7; //msb check
        self.v[var.0 as usize] = self.v[var.0 as usize] << 1;
        self.v[0xF] = flag; //flag atması en son olmalı belki x değeri 0xF olabilir
    }
    //9xy0
    fn sne_vx_vy(&mut self, var: (u8,u8)) {
        //skip not equal Vx,Vy
        if self.v[var.0 as usize] != self.v[var.1 as usize]{
            self.pc+=2;
        }
    }
    //ANNN
    fn ld_i_addr(&mut self, nibbles : (u8,u8,u8) ) {
        //I = nnn
        self.i = comb12(nibbles);
    }
    //Bnnn
    fn jp_v0_addr(&mut self, addr : (u8,u8,u8)){
        //jump to addr+v0
        let sum : u16 = self.v[0] as u16 + comb12(addr);
        self.pc = (self.pc + sum) % self.memory.len() as u16; //index taşması olmasın
    }
    //Cxkk
    fn rnd_vx_byte(&mut self, var : (u8,u8,u8)){
        //Vx = 0-255 arası random sayı & kk
        let num : u8 = fastrand::u8(..);
        self.v[var.0 as usize] = num & comb8((var.1,var.2));
    }
    //DXYN
    fn drw_vx_vy_nib(&mut self, nibbles : (u8,u8,u8)){//I dan başlayarak n byte(ram satırı)lık sprite ı (Vx,Vy) noktasında yazdır, çarpışma olursa VF flag aç (XOR sonucu  1 1 -> 0 olursa)
        //x y n
        //y dikey koordinat x yatay
        let mut j : u8 = 0;
        let x = self.v[nibbles.0 as usize] as usize; //matris sutun
        let mut y = self.v[nibbles.1 as usize] as usize; //matris satır
        self.v[0xF] = 0; //başlamadan flag'i sıfırlayalım
        while j < nibbles.2{ //n kere gezeceğiz (her bir byte için)
            let temp = self.memory[self.i as usize + j as usize]; //elimizde byte var ekranı değiştir
                for bit in (0..8) { // her bit için kontrol yapacağız
                    let mask = 0x80 >> bit; //1000 0000 dan başlayarak maskeyi sağa itiricez ve MSB den başlayarak tüm bitleri gezmiş olacak
                    let sprite = temp & mask; //0 ise false, değil ise true
                    
                    let mut paint :bool = false;
                    if(sprite != 0) {paint = true;}
                    
                    let offset_x = (x+bit) % self.screen[y].len(); //direkt 64 de yazılabilir
                    
                    let pixel = self.screen[y][offset_x];
                    
                    let result = pixel ^ paint; 
                    if result == false && pixel == true {
                        self.v[0xF] = 1;
                    }
                    self.screen[y][offset_x] = result;
            }
            j+=1;
            y = (y+1) % self.screen.len();
        }
        self.print_screen();
    }
    //Ex9E
    fn skp_vx(&mut self, x : u8){
        //skip insturction if Vx==key_pressed(key which is down rn)
    }
    //ExA1
    fn sknp_vx(&mut self, x : u8){
        //skip insturction if Vx==key_not_pressed(key which is not down rn)
    }
    //Fx07
    fn ld_vx_dt(&mut self, x : u8){
        //vx = dt
        
    }
    //Fx0A
    fn ld_vx_k(&mut self, x : u8){
        //store the pressed down key on Vx
    }
    //Fx15
    fn ld_dt_vx(&mut self, x : u8){
        //dt = Vx
    }
    //Fx18
    fn ld_st_vx(&mut self, x : u8){
        //st = Vx   
    }
    //Fx1E
    fn add_i_vx(&mut self, x : u8){
        //I=I+Vx
    }
    //Fx29
    fn ld_f_vx(&mut self, x : u8){
        //I set to digit of sprite corresponding to Vx
    }
    //Fx33
    fn ld_b_vx(&mut self, x : u8){
        //store Vx BCD on I ==> store Vx decimal as, I -> hundreds, I+1 -> tens, I+2-> ones
    }
    //Fx55
    fn ld_i_vx(&mut self, x : u8){
        //store register V0 through Vx in memory starting from I
    }
    //Fx65
    fn ld_vx_i(&mut self, x : u8){
        //read registers V0 through Vx from memory starting from I
    }

}
fn main() {
    let mut chip8 = Chip8::new();

    chip8.read_rom("roms/1-chip8-logo.ch8");
    chip8.print_screen();
    for _ in (1..100){
        let inst = chip8.fetch();
        chip8.decode(inst);
    }
}

fn comb8 (var: (u8,u8)) ->u8 {
    (var.0 << 4 | var.1)
}

fn comb12 (var: (u8,u8,u8)) -> u16 {
    ((var.0 as u16)<< 8) | ((var.1 as u16)<< 4) | (var.2 as u16)
}
