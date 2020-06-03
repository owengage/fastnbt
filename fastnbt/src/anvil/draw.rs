use super::*;
use crate::nbt::{self, Value};
use flate2::read::ZlibDecoder;

pub trait RegionDrawer {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &Chunk);
}

pub struct Chunk {
    heights: Vec<u16>,
    sections: Vec<Option<Section>>,
    biomes: Option<Vec<i32>>,
}

impl Chunk {
    pub fn new(heights: Vec<u16>, sections: Vec<Section>, biomes: Option<Vec<i32>>) -> Self {
        let mut s = Vec::new();
        s.resize_with(16, || None);

        for sec in sections {
            let y = sec.y as usize;
            s[y] = Some(sec);
        }

        // println!("Biomes size: {}", biomes.as_ref().unwrap().len());
        // biomes
        //     .as_ref()
        //     .unwrap()
        //     .as_slice()
        //     .chunks(16)
        //     .for_each(|c| println!("{:?}", c));

        Self {
            heights,
            sections: s,
            biomes,
        }
    }

    pub fn id_of(&self, x: usize, y: usize, z: usize) -> &str {
        let containing_section_y = y / 16;
        let ref sec = self.sections[containing_section_y as usize];

        if let Some(sec) = sec {
            let sec_y = y - sec.y as usize * 16;
            let state_index = (sec_y as usize * 16 * 16) + x * 16 + z;
            let pal_index = sec.states[state_index] as usize;
            &sec.palette[pal_index]
        } else {
            ""
        }
    }

    pub fn height_of(&self, x: usize, z: usize) -> usize {
        self.heights[x * 16 + z] as usize
    }

    pub fn biome_of(&self, x: usize, y: usize, z: usize) -> Option<i32> {
        // FIXME, what's the right way to do this?

        // For biome len of 1024,
        //  it's 4x4x4 sets of blocks stored by z then x then y (+1 moves one in z)
        //  for overworld theres no vertical chunks so it looks like only first 16 values are used.
        // For biome len of 256, it's chunk 1x1 columns stored z then x.

        let biomes = self.biomes.as_ref()?;

        if biomes.len() == 1024 {
            Some(biomes[(x / 4) * 4 + (z / 4)])
        } else {
            Some(biomes[x * 16 + z])
        }
    }
}

#[derive(Debug)]
pub struct Section {
    pub states: Vec<u16>,
    pub palette: Vec<String>,
    pub y: u8,
}

pub struct RegionMap<T> {
    pub data: Vec<T>,
    pub x_region: isize,
    pub z_region: isize,
}

impl<T: Clone> RegionMap<T> {
    pub fn new(x_region: isize, z_region: isize, default: T) -> Self {
        let mut data: Vec<T> = Vec::new();
        data.resize(16 * 16 * 32 * 32, default);
        Self {
            data,
            x_region,
            z_region,
        }
    }

    pub fn chunk_mut(&mut self, x: usize, z: usize) -> &mut [T] {
        let len = 16 * 16;
        let begin = (x * 32 + z) * len; // TODO: x or z ordered?
        &mut self.data[begin..begin + len]
    }

    pub fn chunk(&self, x: usize, z: usize) -> &[T] {
        let len = 16 * 16;
        let begin = (x * 32 + z) * len; // TODO: x or z ordered?
        &self.data[begin..begin + len]
    }
}

pub type Rgb = [u8; 3];

pub struct RegionHeightmapDrawer<'a> {
    map: &'a mut RegionMap<Rgb>,
}

impl<'a> RegionHeightmapDrawer<'a> {
    pub fn new(map: &'a mut RegionMap<Rgb>) -> Self {
        Self { map }
    }
}

impl<'a> RegionDrawer for RegionHeightmapDrawer<'a> {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &Chunk) {
        let data = (*self.map).chunk_mut(xc_rel, zc_rel);

        for z in 0..16 {
            for x in 0..16 {
                const SEA_LEVEL: u16 = 63;
                let height = chunk.heights[x * 16 + z];

                if height <= SEA_LEVEL {
                    data[x * 16 + z] = [height as u8, height as u8, 150];
                } else {
                    let height = (height - 63) * 2;
                    data[x * 16 + z] = [height as u8, 150, height as u8];
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum DrawError {
    ParseAnvil(super::Error),
    ParseNbt(nbt::Error),
    MissingHeightMap,
    InvalidPalette,
}

impl From<nbt::Error> for DrawError {
    fn from(err: nbt::Error) -> DrawError {
        DrawError::ParseNbt(err)
    }
}

impl From<super::Error> for DrawError {
    fn from(err: super::Error) -> DrawError {
        DrawError::ParseAnvil(err)
    }
}

pub type DrawResult<T> = std::result::Result<T, DrawError>;

pub fn parse_region<F: RegionDrawer + ?Sized>(
    mut region: Region<std::fs::File>,
    draw_to: &mut F,
) -> DrawResult<()> {
    let mut offsets = Vec::<ChunkLocation>::new();

    for x in 0..32 {
        for z in 0..32 {
            let loc = region.chunk_location(x, z)?;

            // 0,0 chunk location means the chunk isn't present.
            // cannot decide if this means we should return an error from chunk_location() or not.
            if loc.begin_sector != 0 && loc.sector_count != 0 {
                offsets.push(loc);
            }
        }
    }

    offsets.sort_by(|o1, o2| o1.begin_sector.cmp(&o2.begin_sector));

    for off in offsets {
        let mut buf = vec![0u8; off.sector_count * SECTOR_SIZE];
        region.load_chunk(&off, &mut buf[..])?;

        let chunk = parse_chunk(buf.as_slice());

        match chunk {
            Ok(chunk) => draw_to.draw(off.x, off.z, &chunk),
            Err(DrawError::MissingHeightMap) => {} // skip this chunk.
            Err(e) => return Err(e),
        };
    }

    Ok(())
}

pub fn parse_chunk(data: &[u8]) -> DrawResult<Chunk> {
    let meta = super::ChunkMeta::new(data)?;

    let buf = &data[5..];
    let decoder = match meta.compression_scheme {
        CompressionScheme::Zlib => ZlibDecoder::new(buf),
        _ => panic!("unknown compression scheme (gzip?)"),
    };

    let mut parser = nbt::Parser::new(decoder);

    nbt::find_compound(&mut parser, Some("Level"))?;

    let mut sections = Vec::new();
    let mut heightmap: Option<Vec<u16>> = None;
    let mut biomes = None;

    loop {
        match parser.next()? {
            Value::List(Some(ref name), _, n) if name == "Sections" => {
                for _ in 0..n {
                    if let Some(section) = process_section(&mut parser)? {
                        sections.push(section);
                    }
                }
            }
            Value::IntArray(Some(ref name), b) if name == "Biomes" => {
                biomes = Some(b);
            }
            Value::Compound(Some(name)) if name == "Heightmaps" => {
                heightmap = process_heightmap(&mut parser)?;
            }
            Value::Compound(_) => {
                // don't enter another compound.
                nbt::skip_compound(&mut parser)?;
            }
            Value::CompoundEnd => {
                if heightmap == None {
                    return Err(DrawError::MissingHeightMap);
                }

                return Ok(Chunk::new(heightmap.unwrap(), sections, biomes));
            }
            _ => {}
        }
    }
}

fn process_palette<R: Read>(parser: &mut nbt::Parser<R>, size: usize) -> DrawResult<Vec<String>> {
    let mut names = Vec::<String>::new();

    for _ in 0..size {
        let v = parser.next().map_err(|_| Error::InsufficientData)?; // start compound

        match v {
            nbt::Value::Compound(None) => {}
            _ => return Err(DrawError::InvalidPalette),
        }

        // Find name, skipping the rest of the stuff.
        loop {
            let v = parser.next()?;

            match v {
                Value::Compound(_) => {
                    nbt::skip_compound(parser)?;
                } // if we find a nested compound, skip it.
                Value::String(Some(name), str) if name == "Name" => {
                    names.push(str);
                    break;
                }
                Value::CompoundEnd => return Err(DrawError::InvalidPalette), // didn't find name.
                Value::List(_, _, _) => return Err(DrawError::InvalidPalette),
                _ => {}
            }
        }

        // Loop until we find the End.
        nbt::skip_compound(parser)?;
    }

    Ok(names)
}

fn process_section<R: Read>(mut parser: &mut nbt::Parser<R>) -> DrawResult<Option<Section>> {
    nbt::find_compound(&mut parser, None)?;
    let mut states = Vec::<i64>::new();
    let mut palette = Vec::<String>::new();
    let mut y = None;

    loop {
        let value = parser.next()?;

        match value {
            Value::List(Some(ref name), _, n) if name == "Palette" => {
                palette = process_palette(&mut parser, n as usize)?;
            }
            Value::LongArray(Some(ref name), s) if name == "BlockStates" => {
                //println!("{:?}", s);
                states = s;
            }
            Value::Byte(Some(ref name), b) if name == "Y" => {
                y = Some(b);
            }
            Value::Compound(_) => {
                // don't enter another compound.
                nbt::skip_compound(&mut parser)?;
            }
            Value::CompoundEnd => {
                // Sections might be empty if there are no blocks
                // Also see sections with y = -1 with no data.
                return Ok(None);
            }
            _ => {}
        }

        // Do we have a palette and blockstate?
        if states.len() > 0 && palette.len() > 0 && y.is_some() {
            let expanded = bits::expand_blockstates(&states[..], palette.len());
            nbt::skip_compound(&mut parser)?;
            return Ok(Some(Section {
                states: expanded,
                palette,
                y: y.unwrap() as u8, // know y.is_some() above.
            }));
        }
    }
}

fn process_heightmap<R: Read>(mut parser: &mut nbt::Parser<R>) -> DrawResult<Option<Vec<u16>>> {
    loop {
        match parser.next()? {
            Value::LongArray(Some(ref name), data) if name == "WORLD_SURFACE" => {
                nbt::skip_compound(&mut parser)?;
                return Ok(Some(bits::expand_heightmap(data.as_slice())));
            }
            Value::Compound(_) => {
                // don't enter another compound.
                nbt::skip_compound(&mut parser)?;
            }
            Value::CompoundEnd => {
                // No heightmap found, it happens.
                return Ok(None);
            }
            _ => {}
        }
    }
}
