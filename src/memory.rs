pub type Addr = u16;
pub enum IoReg {
    Lcdc = 0xff40,
    Bcps = 0xff68,
    Bcpd = 0xff69,
}

impl From<IoReg> for u8 {
    fn from(value: IoReg) -> Self {
        ((value as u16) & 0x00ff) as u8
    }
}