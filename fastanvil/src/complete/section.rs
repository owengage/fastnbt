use crate::{java, Block};

pub struct Section {
    block_palette: Vec<Block>,

    //could be [] because of fix size
    //This will increase in x, then z, then y.
    blocks: Option<Vec<u16>>,
}

impl Section {
    pub fn from_current_section(current_tower: &java::Section) -> Self {
        let mut blocks = None;

        if let Some(iter) = current_tower.block_states.try_iter_indices() {
            let mut blocks_indexes = vec![];

            for block_index in iter {
                blocks_indexes.push(block_index as u16);
            }

            blocks = Some(blocks_indexes);
        }

        Section {
            block_palette: Vec::from(current_tower.block_states.palette()),
            blocks,
        }
    }

    pub fn block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        return match &self.blocks {
            None => Some(self.block_palette.get(0).unwrap()),
            Some(blocks) => {
                let index = y * (16 * 16) + z * 16 + x;

                self.block_palette.get(*blocks.get(index).unwrap() as usize)
            }
        };
    }
}
