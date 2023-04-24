pub struct Parameter<T> {
    value: T,
    previous_value: T,
}

impl<T> Parameter<T>
where
    T: Copy + PartialEq,
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            previous_value: value,
        }
    }

    pub fn get(&self) -> T {
        self.value
    }

    #[allow(dead_code)]
    pub fn get_previous(&self) -> T {
        self.previous_value
    }

    pub fn set(&mut self, value: T) {
        self.previous_value = self.value;
        self.value = value;
    }

    pub fn has_changed(&self) -> bool {
        self.value != self.previous_value
    }
}
