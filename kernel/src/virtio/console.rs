use super::*;

pub struct ConsoleDevice {
    
}
impl VirtIODevice for ConsoleDevice {
    fn handle_int(&mut self) {
        todo!()
    }
}