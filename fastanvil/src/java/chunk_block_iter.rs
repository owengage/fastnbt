use crate::{Block, BlockData, Chunk, CurrentJavaChunk, JavaChunk, Section, SectionTower, StatesIter};

pub struct CurrentChunkBlockIter<'a> {
    section_tower: &'a SectionTower<Section>,
    current_block_states: Option<&'a BlockData<Block>>,
    current_section_iter: Option<StatesIter<'a>>,
    next_y: isize,

    blocks_to_fill: usize,
}

impl<'a> CurrentChunkBlockIter<'a> {
    pub fn new(chunk: &'a mut CurrentJavaChunk) -> Self {
        let mut thing = Self {
            next_y: -64,
            section_tower: chunk.sections.as_ref().unwrap(),
            current_section_iter: None,
            current_block_states: None,
            blocks_to_fill: 0,
        };

        thing.init();

        return thing;
    }

    fn init(&mut self) {

        self.current_block_states = Some(
            &self
                .section_tower
                .get_section_for_y(self.next_y)
                .unwrap()
                .block_states,
        );

        match self.current_block_states.unwrap().try_iter_indices() {
            None => self.blocks_to_fill = 16 * 16 * 16,
            Some(some) => self.current_section_iter = Some(some),
        }

        self.next_y += 16;
    }
}

impl<'a> Iterator for CurrentChunkBlockIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.blocks_to_fill > 0 {
            self.blocks_to_fill -= 1;

            return Some(
                self.current_block_states
                    .unwrap()
                    .palette()
                    .get(0)
                    .unwrap()
                    .clone(),
            );
        }

        return match self.current_section_iter.as_mut().unwrap().next() {
            None => {
                if self.next_y >= self.section_tower.y_max() {
                    return None;
                };

                self.init();

                self.next()
            }
            Some(index) => {
                Some(
                    self.current_block_states
                        .unwrap()
                        .palette()
                        .get(index)
                        .unwrap()
                        .clone(),
                )
            }
        }
    }
}


pub struct ChunkBlockIter<'a> {
    chunk: &'a JavaChunk,

    x: usize,
    y: isize,
    z: usize,
}

impl<'a> ChunkBlockIter<'a> {
    pub fn new(chunk: &'a mut JavaChunk) -> Self {
        Self {
            chunk,
            x: 0,
            y: -64,
            z: 0,
        }
    }
}

impl<'a> Iterator for ChunkBlockIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.x += 1;

        if self.x >= 16 {
            self.x = 0;
            self.z += 1;

            if self.z >= 16 {
                self.z = 0;
                self.y += 1;

                if self.y >= 320 {
                    return None;
                }
            }
        }

        return match self.chunk.block(self.x, self.y, self.z) {
            None => None,
            Some(block) => Some(block.clone()),
        };
    }
}

