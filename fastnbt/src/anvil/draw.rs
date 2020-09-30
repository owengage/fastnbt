use super::*;
use crate::nbt::{self};
use crate::nbt2;
use flate2::read::ZlibDecoder;
use types::Chunk;

pub trait RegionDrawer {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &mut Chunk);
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

// pub struct RegionHeightmapDrawer<'a> {
//     map: &'a mut RegionMap<Rgb>,
// }

// impl<'a> RegionHeightmapDrawer<'a> {
//     pub fn new(map: &'a mut RegionMap<Rgb>) -> Self {
//         Self { map }
//     }
// }

// impl<'a> RegionDrawer for RegionHeightmapDrawer<'a> {
//     fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &Chunk) {
//         let data = (*self.map).chunk_mut(xc_rel, zc_rel);

//         for z in 0..16 {
//             for x in 0..16 {
//                 const SEA_LEVEL: u16 = 63;
//                 let height = chunk.heights[x * 16 + z];

//                 if height <= SEA_LEVEL {
//                     data[x * 16 + z] = [height as u8, height as u8, 150];
//                 } else {
//                     let height = (height - 63) * 2;
//                     data[x * 16 + z] = [height as u8, 150, height as u8];
//                 }
//             }
//         }
//     }
// }

#[derive(Debug)]
pub enum DrawError {
    ParseAnvil(super::Error),
    ParseNbt(nbt::Error),
    ParseNbt2(nbt2::error::Error),
    IO(std::io::Error),
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

impl From<crate::nbt2::error::Error> for DrawError {
    fn from(err: crate::nbt2::error::Error) -> DrawError {
        DrawError::ParseNbt2(err)
    }
}

impl From<std::io::Error> for DrawError {
    fn from(err: std::io::Error) -> Self {
        DrawError::IO(err)
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
        let mut buf = vec![0; off.sector_count * SECTOR_SIZE];
        region.load_chunk(&off, &mut buf[..])?;

        let data = decompress_chunk(buf.as_slice())?;
        let chunk: DrawResult<types::Chunk> = Ok(nbt2::de::from_bytes(data.as_slice())?);

        match chunk {
            Ok(mut chunk) => draw_to.draw(off.x, off.z, &mut chunk),
            Err(DrawError::MissingHeightMap) => {} // skip this chunk.
            Err(e) => return Err(e),
        };
    }

    Ok(())
}

pub fn decompress_chunk(data: &[u8]) -> DrawResult<Vec<u8>> {
    let meta = super::ChunkMeta::new(data)?;

    let buf = &data[5..];
    let mut decoder = match meta.compression_scheme {
        CompressionScheme::Zlib => ZlibDecoder::new(buf),
        _ => panic!("unknown compression scheme (gzip?)"),
    };

    // Try to reduce number of allocations, decompressed is going to be at least the size of
    // the original data, you'd hope.
    let mut buf = Vec::with_capacity(buf.len());
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

// fn process_palette<R: Read>(parser: &mut nbt::Parser<R>, size: usize) -> DrawResult<Vec<String>> {
//     let mut names = Vec::<String>::new();

//     for _ in 0..size {
//         let v = parser.next().map_err(|_| Error::InsufficientData)?; // start compound

//         match v {
//             nbt::Value::Compound(None) => {}
//             _ => return Err(DrawError::InvalidPalette),
//         }

//         // Find name, skipping the rest of the stuff.
//         loop {
//             let v = parser.next()?;

//             match v {
//                 Value::Compound(_) => {
//                     nbt::skip_compound(parser)?;
//                 } // if we find a nested compound, skip it.
//                 Value::String(Some(name), str) if name == "Name" => {
//                     names.push(str);
//                     break;
//                 }
//                 Value::CompoundEnd => return Err(DrawError::InvalidPalette), // didn't find name.
//                 Value::List(_, _, _) => return Err(DrawError::InvalidPalette),
//                 _ => {}
//             }
//         }

//         // Loop until we find the End.
//         nbt::skip_compound(parser)?;
//     }

//     Ok(names)
// }

// fn process_section<R: Read>(mut parser: &mut nbt::Parser<R>) -> DrawResult<Option<Section>> {
//     nbt::find_compound(&mut parser, None)?;
//     let mut states = Vec::<i64>::new();
//     let mut palette = Vec::<String>::new();
//     let mut y = None;

//     loop {
//         let value = parser.next()?;

//         match value {
//             Value::List(Some(ref name), _, n) if name == "Palette" => {
//                 palette = process_palette(&mut parser, n as usize)?;
//             }
//             Value::LongArray(Some(ref name), s) if name == "BlockStates" => {
//                 states = s;
//             }
//             Value::Byte(Some(ref name), b) if name == "Y" => {
//                 y = Some(b);
//             }
//             Value::Compound(_) => {
//                 // don't enter another compound.
//                 nbt::skip_compound(&mut parser)?;
//             }
//             Value::CompoundEnd => {
//                 // Sections might be empty if there are no blocks
//                 // Also see sections with y = -1 with no data.
//                 return Ok(None);
//             }
//             _ => {}
//         }

//         // Do we have a palette and blockstate?
//         if states.len() > 0 && palette.len() > 0 && y.is_some() {
//             let expanded = bits::expand_blockstates(&states[..], palette.len());
//             nbt::skip_compound(&mut parser)?;
//             return Ok(Some(Section {
//                 states: expanded,
//                 palette,
//                 y: y.unwrap() as u8, // know y.is_some() above.
//             }));
//         }
//     }
// }

// fn process_heightmap<R: Read>(mut parser: &mut nbt::Parser<R>) -> DrawResult<Option<Vec<u16>>> {
//     loop {
//         match parser.next()? {
//             Value::LongArray(Some(ref name), data) if name == "WORLD_SURFACE" => {
//                 nbt::skip_compound(&mut parser)?;
//                 return Ok(Some(bits::expand_heightmap(data.as_slice())));
//             }
//             Value::Compound(_) => {
//                 // don't enter another compound.
//                 nbt::skip_compound(&mut parser)?;
//             }
//             Value::CompoundEnd => {
//                 // No heightmap found, it happens.
//                 return Ok(None);
//             }
//             _ => {}
//         }
//     }
// }
