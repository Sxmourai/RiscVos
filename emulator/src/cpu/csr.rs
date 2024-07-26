use super::uguest;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CsrID(u16); // 12 bits
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CsrValue(pub uguest);

impl CsrID {
    pub fn new(id: u16) -> Self {
        assert!(id<(1<<12));
        Self(id)
    }
    pub fn get(self) -> u16 {self.0}
}
