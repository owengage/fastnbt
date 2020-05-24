use super::*;

pub struct Builder {
    payload: Vec<u8>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            payload: Vec::new(),
        }
    }

    pub fn tag(mut self, t: Tag) -> Self {
        self.payload.push(t as u8);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        let len_bytes = &(name.len() as u16).to_be_bytes()[..];
        self.payload.extend_from_slice(len_bytes);
        self.payload.extend_from_slice(name.as_bytes());
        self
    }

    pub fn string_payload(self, s: &str) -> Self {
        self.name(s)
    }

    pub fn byte_payload(mut self, b: i8) -> Self {
        self.payload.push(b as u8);
        self
    }

    pub fn byte_array_payload(mut self, bs: &[i8]) -> Self {
        for b in bs {
            self.payload.push(*b as u8);
        }
        self
    }

    pub fn short_payload(mut self, i: i16) -> Self {
        self.payload.extend_from_slice(&i.to_be_bytes()[..]);
        self
    }

    pub fn int_payload(mut self, i: i32) -> Self {
        self.payload.extend_from_slice(&i.to_be_bytes()[..]);
        self
    }

    pub fn int_array_payload(mut self, is: &[i32]) -> Self {
        for i in is {
            self = self.int_payload(*i);
        }
        self
    }

    pub fn long_payload(mut self, i: i64) -> Self {
        self.payload.extend_from_slice(&i.to_be_bytes()[..]);
        self
    }

    pub fn long_array_payload(mut self, is: &[i64]) -> Self {
        for i in is {
            self = self.long_payload(*i);
        }
        self
    }

    pub fn float_payload(mut self, f: f32) -> Self {
        self.payload.extend_from_slice(&f.to_be_bytes()[..]);
        self
    }

    pub fn double_payload(mut self, f: f64) -> Self {
        self.payload.extend_from_slice(&f.to_be_bytes()[..]);
        self
    }

    pub fn build(self) -> Vec<u8> {
        self.payload
    }
}
