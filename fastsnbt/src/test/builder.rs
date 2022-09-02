pub struct Builder {
    payload: Vec<u8>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            payload: vec![b'{'],
        }
    }

    pub fn build(mut self) -> Vec<u8> {
        self.payload.push(b'}');
        self.payload
    }

    pub fn unquoted_name(mut self, name: &str) -> Self {
        let name = cesu8::to_java_cesu8(name);
        self.payload.extend_from_slice(&name);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        let name = cesu8::to_java_cesu8(name);
        self.payload.push(b'"');
        self.payload.extend_from_slice(&name);
        self.payload.push(b'"');
        self
    }

    pub fn single_quoted_name(mut self, name: &str) -> Self {
        let name = cesu8::to_java_cesu8(name);
        self.payload.push(b'\'');
        self.payload.extend_from_slice(&name);
        self.payload.push(b'\'');
        self
    }

    pub fn byte(mut self, name: &str, b: u8) -> Self {
        self = self.name(name);
        self.payload.push(b':');
        self.payload.extend_from_slice(b.to_string().as_bytes());
        self.payload.push(b'B');
        self
    }

    pub fn byte_lowercase(mut self, b: u8) -> Self {
        self.payload.extend_from_slice(b.to_string().as_bytes());
        self.payload.push(b'b');
        self
    }

    // pub fn start_compound(self, name: &str) -> Self {
    //     self.payload.push(b'{');
    //     self.name(name)
    // }
}
