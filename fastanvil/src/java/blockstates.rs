use std::cell::{Cell, RefCell};

use serde::Deserialize;

use crate::{bits_per_block, PackedBits};

#[derive(Debug)]
pub struct Blockstates {
    done: Cell<bool>,
    unpacked: RefCell<[u16; 16 * 16 * 16]>,
    packed: PackedBits,
}

impl Blockstates {
    #[inline(always)]
    pub fn state(&self, x: usize, sec_y: usize, z: usize, pal_len: usize) -> usize {
        // 🤮 This is a very hot function, so the ugly is worth the speed.
        if !self.done.get() {
            let bits_per_item = bits_per_block(pal_len);
            let mut buf = self.unpacked.borrow_mut();
            let mut buf = buf.as_mut();

            self.packed.unpack_blockstates(bits_per_item, &mut buf);
            self.done.replace(true);
        }

        let state_index = (sec_y * 16 * 16) + z * 16 + x;

        // We *know* unpacked is filled in because we just made it above.
        self.unpacked.borrow().as_ref()[state_index] as usize
    }
}

impl<'de> Deserialize<'de> for Blockstates {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let packed: PackedBits = Deserialize::deserialize(d)?;
        Ok(Self {
            done: Cell::new(false),
            packed,
            unpacked: RefCell::new([0; 16 * 16 * 16]),
        })
    }
}
