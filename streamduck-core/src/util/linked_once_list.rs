use std::cell::OnceCell;

#[derive(Default, Debug)]
pub struct LinkedOnceList<T> {
    element: OnceCell<T>,
    next: OnceCell<Box<LinkedOnceList<T>>>
}

impl<T> LinkedOnceList<T> {
    pub fn new() -> LinkedOnceList<T> {
        Self {
            element: OnceCell::new(),
            next: OnceCell::new()
        }
    }

    pub fn new_boxed() -> Box<LinkedOnceList<T>> {
        Box::new(Self::new())
    }

    pub fn push(&self, value: T) -> &T {
        let Err(value) = self.element.set(value) else {
            return self.element.get().unwrap()
        };

        let mut next = self.next.get_or_init(Self::new_boxed);
        let mut value_to_put = value;

        loop {
            let Err(value) = next.element.set(value_to_put) else {
                return next.element.get().unwrap()
            };

            value_to_put = value;
            next = next.next.get_or_init(Self::new_boxed)
        }
    }

    pub fn extend<I: IntoIterator<Item=T>>(&self, iter: I) {
        let mut tail = self;

        while tail.element.get().is_some() {
            tail = tail.next.get_or_init(Self::new_boxed)
        }

        let mut current = tail;
        for item in iter {
            let Err(item) = current.element.set(item) else {
                continue;
            };

            let next = current.next.get_or_init(Self::new_boxed);
            next.element.set(item).ok().unwrap();

            current = next.as_ref();
        }
    }

    pub fn iter(&self) -> LinkedOnceIterator<'_, T> {
        LinkedOnceIterator {
            current: Some(self)
        }
    }
}

impl<T> FromIterator<T> for LinkedOnceList<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let root = LinkedOnceList::new();
        root.extend(iter);
        root
    }
}

pub struct LinkedOnceIterator<'a, T> {
    current: Option<&'a LinkedOnceList<T>>
}

impl<'a, T> Iterator for LinkedOnceIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = current.next.get().map(|a| a.as_ref());
        current.element.get()
    }
}