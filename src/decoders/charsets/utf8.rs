use std::char::REPLACEMENT_CHARACTER;

use crate::decoders::Writer;

enum Utf8State {
    Start,
    Shift12,
    Shift6,
    Shift0,
}

pub struct Utf8Decoder {
    state: Utf8State,
    char: u32,
    result: String,
}

impl Writer for Utf8Decoder {
    fn write_byte(&mut self, byte: &u8) -> bool {
        match self.state {
            Utf8State::Start => {
                if *byte < 0x80 {
                    self.result
                        .push(unsafe { char::from_u32_unchecked(*byte as u32) });
                } else if (*byte & 0xe0) == 0xc0 {
                    self.char = (*byte as u32 & 0x1f) << 6;
                    self.state = Utf8State::Shift0;
                } else if (*byte & 0xf0) == 0xe0 {
                    self.char = (*byte as u32 & 0x0f) << 12;
                    self.state = Utf8State::Shift6;
                } else if (*byte & 0xf8) == 0xf0 && (*byte <= 0xf4) {
                    self.char = (*byte as u32 & 0x07) << 18;
                    self.state = Utf8State::Shift12;
                } else {
                    self.result.push(REPLACEMENT_CHARACTER);
                }
            }
            Utf8State::Shift12 => {
                self.char |= (*byte as u32 & 0x3f) << 12;
                self.state = Utf8State::Shift6;
            }
            Utf8State::Shift6 => {
                self.char |= (*byte as u32 & 0x3f) << 6;
                self.state = Utf8State::Shift0;
            }
            Utf8State::Shift0 => {
                self.char |= *byte as u32 & 0x3f;
                self.state = Utf8State::Start;
                self.result
                    .push(char::from_u32(self.char).unwrap_or(REPLACEMENT_CHARACTER));
                self.char = 0;
            }
        }
        true
    }

    fn get_string(&mut self) -> Option<String> {
        if !self.result.is_empty() {
            Some(std::mem::take(&mut self.result))
        } else {
            None
        }
    }

    fn get_bytes(&mut self) -> Option<Box<[u8]>> {
        None
    }
}

impl Utf8Decoder {
    pub fn new(capacity: usize) -> Utf8Decoder {
        Utf8Decoder {
            result: String::with_capacity(capacity),
            state: Utf8State::Start,
            char: 0,
        }
    }

    pub fn get_utf8(capacity: usize) -> Box<dyn Writer> {
        Box::new(Utf8Decoder::new(capacity))
    }
}

#[cfg(test)]
mod tests {
    use crate::decoders::Writer;

    use super::Utf8Decoder;

    #[test]
    fn decode_utf8() {
        let inputs = [
            (b"Lorem ipsum".to_vec(), "Lorem ipsum"),
            (b"Th\xc3\xads \xc3\xads v\xc3\xa1l\xc3\xadd \xc3\x9aTF8".to_vec(), "Thís ís válíd ÚTF8"),
            (b"\xe3\x83\x8f\xe3\x83\xad\xe3\x83\xbc\xe3\x83\xbb\xe3\x83\xaf\xe3\x83\xbc\xe3\x83\xab\xe3\x83\x89".to_vec(), "ハロー・ワールド"),
            (b"\xec\x95\x88\xeb\x85\x95\xed\x95\x98\xec\x84\xb8\xec\x9a\x94 \xec\x84\xb8\xea\xb3\x84".to_vec(), "안녕하세요 세계"),
            (b"love: \xe2\x9d\xa4\xef\xb8\x8f".to_vec(), "love: ❤️"),
            (b"\xec \x95\x88 \xeb\x85\x95 \xed\x95\x98\xec\x84\xb8 \xec\x9a\x94 \xec\x84\xb8\xea\xb3 \x84".to_vec(), "정� 녕 하세 요 세고�"),
        ];

        for input in inputs {
            let mut parser = Utf8Decoder::new(10);
            parser.write_bytes(&input.0);

            assert_eq!(parser.get_string().unwrap(), input.1);
        }
    }
}