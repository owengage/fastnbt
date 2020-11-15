use fastanvil::{biome::Biome, Chunk, ChunkRender, RegionMap, Rgba};

pub struct RegionBiomeDrawer<'a> {
    pub map: &'a mut RegionMap<Rgba>,
}

impl<'a> ChunkRender for RegionBiomeDrawer<'a> {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &mut Chunk) {
        let data = (*self.map).chunk_mut(xc_rel, zc_rel);

        for z in 0..16 {
            for x in 0..16 {
                let y = chunk.height_of(x, z).unwrap_or(64);
                let biome = chunk.biome_of(x, y, z).unwrap_or(Biome::TheVoid);

                // TODO:  If material is grass block (and others), we need to colour it based on biome.
                let colour = match biome {
                    Biome::Ocean => [0, 0, 200, 255],
                    Biome::DeepOcean => [0, 0, 150, 255],
                    Biome::ColdOcean
                    | Biome::DeepColdOcean
                    | Biome::DeepFrozenOcean
                    | Biome::DeepLukewarmOcean
                    | Biome::DeepWarmOcean
                    | Biome::FrozenOcean
                    | Biome::LukewarmOcean
                    | Biome::WarmOcean => [0, 0, 255, 255],
                    Biome::River | Biome::FrozenRiver => [0, 0, 255, 255],
                    Biome::Beach => [100, 50, 50, 255],

                    b => {
                        let b: i32 = b.into();
                        [b as u8, b as u8, b as u8, 255]
                    }
                };

                let pixel = &mut data[x * 16 + z];
                *pixel = colour;
            }
        }
    }

    fn draw_invalid(&mut self, _xc_rel: usize, _zc_rel: usize) {}
}
