use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug)]
pub struct Shared<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> Hash for Shared<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.borrow().hash(state)
    }
}

impl<T> PartialEq for Shared<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Eq for Shared<T> where T: PartialEq {}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Shared {
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

impl<T> Clone for SharedImmutable<T> {
    fn clone(&self) -> Self {
        SharedImmutable {
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
        SharedImmutable {
            inner: Rc::new(value),
        }
    }
}
