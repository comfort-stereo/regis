use std::cell::{Ref, RefCell, RefMut};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct SharedMutable<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> From<T> for SharedMutable<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T> Clone for SharedMutable<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> Hash for SharedMutable<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.borrow().hash(state)
    }
}

impl<T> PartialEq for SharedMutable<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Eq for SharedMutable<T> where T: PartialEq {}

impl<T> SharedMutable<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

#[derive(Debug)]
pub struct SharedImmutable<T> {
    inner: Rc<T>,
}

impl<T> From<T> for SharedImmutable<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T> Deref for SharedImmutable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Clone for SharedImmutable<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> Hash for SharedImmutable<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<T> PartialEq for SharedImmutable<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Eq for SharedImmutable<T> where T: PartialEq {}

impl<T> SharedImmutable<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value),
        }
    }
}
