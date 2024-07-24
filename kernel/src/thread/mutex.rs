pub struct Mutex<T> {
    inner: T,
}
impl<'a, T> Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Self {inner}
    }
    pub fn lock(&'a mut self) -> MutexGuard<'a, T> {
        MutexGuard { inner: &mut self.inner }
    }
}


pub struct MutexGuard<'a, T> {
    inner: &'a mut T,
}
impl<'a, T> core::ops::Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'a, T> core::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}