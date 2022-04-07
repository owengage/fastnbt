use flate2::read::ZlibDecoder;
use std::convert::TryFrom;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_enum::TryFromPrimitive;

use crate::{CCoord, JavaChunk};
use crate::{Error, Result};

/// the size in bytes of a 'sector' in a region file. Sectors are Minecraft's size unit
/// for chunks. For example, a chunk might be `3 * SECTOR_SIZE` bytes. The
/// actual compressed bytes of a chunk may be smaller and the exact value is
/// tracking in the chunk header.
pub(crate) const SECTOR_SIZE: usize = 4096;

/// the size of the region file header.
pub(crate) const REGION_HEADER_SIZE: usize = 2 * SECTOR_SIZE;

/// size of header for each chunk in the region file. This header proceeds the
/// compressed chunk data.
pub(crate) const CHUNK_HEADER_SIZE: usize = 5;

// pub trait Region {
//     /// Load the chunk at the given chunk coordinates, ie 0..32 for x and z.
//     /// Implmentations do not need to be concerned with caching chunks they have
//     /// loaded, this will be handled by the types using the region.
//     fn chunk(&mut self, x: CCoord, z: CCoord) -> Option<JavaChunk>;
// }

pub trait RegionRead {
    fn read_chunk(&mut self, x: usize, z: usize) -> Result<Vec<u8>> {
        // Metadata encodes the length in bytes and the compression type
        let (scheme, compressed) = self.read_compressed_chunk(x, z)?;
        let compressed = Cursor::new(compressed);

        let mut decoder = match scheme {
            CompressionScheme::Zlib => ZlibDecoder::new(compressed),
            _ => panic!("unknown compression scheme (gzip?)"),
        };

        let mut outbuf = Vec::new();
        // read the whole Chunk
        decoder.read_to_end(&mut outbuf)?;
        Ok(outbuf)
    }

    fn read_compressed_chunk(&mut self, x: usize, z: usize)
        -> Result<(CompressionScheme, Vec<u8>)>;
}

pub trait RegionWrite {
    /// Low level method. Write a chunk to the region file that has already been
    /// appropriately compressed for storage.
    fn write_compressed_chunk(
        &mut self,
        x: usize,
        z: usize,
        scheme: CompressionScheme,
        compressed_chunk: &[u8],
    ) -> Result<()>;
}

/// A Minecraft Region.
pub struct RegionBuffer<S> {
    data: S,
    // last offset is always the next valid place to write a chunk.
    offsets: Vec<u64>,
}

impl<S> RegionBuffer<S>
where
    S: Read + Write + Seek,
{
    pub fn new_empty(mut buf: S) -> Result<Self> {
        buf.rewind()?;
        buf.write_all(&[0; REGION_HEADER_SIZE])?;

        Ok(Self {
            data: buf,
            offsets: vec![2], // 2 is the end of the header
        })
    }

    /// Return the inner buffer used. The buffer is rewound to the beginning.
    pub fn into_inner(mut self) -> io::Result<S> {
        self.data.rewind()?;
        Ok(self.data)
    }

    pub(crate) fn header_pos(&self, x: usize, z: usize) -> u64 {
        (4 * ((x % 32) + (z % 32) * 32)) as u64
    }

    pub(crate) fn info(&mut self, x: usize, z: usize) -> io::Result<(u64, u64)> {
        self.data.seek(SeekFrom::Start(self.header_pos(x, z)))?;

        let mut buf = [0u8; 4];
        self.data.read_exact(&mut buf[..])?;

        let mut off = 0u64;
        off |= (buf[0] as u64) << 16;
        off |= (buf[1] as u64) << 8;
        off |= buf[2] as u64;
        let count = buf[3] as u64;

        Ok((off, count))
    }

    fn set_chunk(&mut self, offset: u64, scheme: CompressionScheme, chunk: &[u8]) -> Result<()> {
        self.data
            .seek(SeekFrom::Start(offset * SECTOR_SIZE as u64))?;

        self.data.write_all(&self.chunk_meta(
            chunk.len() as u32, // doesn't include header size
            scheme,
        ))?;

        self.data.write_all(chunk)?;
        Ok(())
    }

    pub(crate) fn set_header(
        &mut self,
        x: usize,
        z: usize,
        offset: u64,
        new_sector_count: usize,
    ) -> Result<()> {
        if new_sector_count > 255 {
            return Err(Error::ChunkTooLarge);
        }

        let mut buf = [0u8; 4];
        buf[0] = ((offset & 0xFF0000) >> 16) as u8;
        buf[1] = ((offset & 0x00FF00) >> 8) as u8;
        buf[2] = (offset & 0x0000FF) as u8;
        buf[3] = new_sector_count as u8; // TODO, what if it doesn't fit.

        // seek to header
        self.data.seek(SeekFrom::Start(self.header_pos(x, z)))?;
        self.data.write_all(&buf)?;
        Ok(())
    }

    fn chunk_meta(&self, compressed_chunk_size: u32, scheme: CompressionScheme) -> [u8; 5] {
        // let mut buf = &data[..5];
        // let len = buf.read_u32::<BigEndian>()?;
        // let scheme = buf.read_u8()?;
        // let scheme = CompressionScheme::try_from(scheme).map_err(|_|
        // Error::InvalidChunkMeta)?;
        let mut buf = [0u8; 5];
        let mut c = Cursor::new(buf.as_mut_slice());

        // The given size is the compressed chunk alone, but the size written to
        // disk includes the byte representing the compression scheme, so +1.
        c.write_u32::<BigEndian>(compressed_chunk_size + 1).unwrap();
        c.write_u8(match scheme {
            CompressionScheme::Gzip => 1,
            CompressionScheme::Zlib => 2,
            CompressionScheme::Uncompressed => 3,
        })
        .unwrap();

        buf
    }
}

impl<S> RegionRead for RegionBuffer<S>
where
    S: Read + Write + Seek,
{
    fn read_compressed_chunk(
        &mut self,
        x: usize,
        z: usize,
    ) -> Result<(CompressionScheme, Vec<u8>)> {
        if x >= 32 || z >= 32 {
            return Err(Error::InvalidOffset(x as isize, z as isize));
        }

        let (off, count) = self.info(x, z)?;

        if off == 0 && count == 0 {
            Err(Error::ChunkNotFound)
        } else {
            self.data.seek(SeekFrom::Start(off * SECTOR_SIZE as u64))?;

            let mut buf = [0u8; 5];
            self.data.read_exact(&mut buf)?;
            let metadata = ChunkMeta::new(&buf)?;

            let mut compressed_chunk = vec![0; metadata.compressed_len as usize];
            self.data.read_exact(&mut compressed_chunk)?;

            Ok((metadata.compression_scheme, compressed_chunk))
        }
    }
}

impl<S> RegionWrite for RegionBuffer<S>
where
    S: Seek + Write + Read,
{
    fn write_compressed_chunk(
        &mut self,
        x: usize,
        z: usize,
        scheme: CompressionScheme,
        chunk: &[u8],
    ) -> Result<()> {
        let (offset, count) = self.info(x, z)?;
        let required_sectors = unstable_div_ceil(CHUNK_HEADER_SIZE + chunk.len(), SECTOR_SIZE);

        if offset == 0 && count == 0 {
            // chunk does not exist in the region yet.
            let offset = *self.offsets.last().expect("offset should always exist");

            // add a new offset representing the new 'end' of the current region file.
            self.offsets.push(offset + required_sectors as u64);
            self.set_chunk(offset, scheme, chunk)?;
            self.set_header(x, z, offset, required_sectors)?;
        } else {
            // chunk already exists in the region file, need to update it.
            let i = self.offsets.binary_search(&offset).unwrap();
            let start_offset = self.offsets[i];
            let end_offset = self.offsets[i + 1];
            let available_sectors = (end_offset - start_offset) as usize;

            if required_sectors <= available_sectors {
                // we fit in the current gap in the file.
                self.set_chunk(start_offset, scheme, chunk)?;
                self.set_header(x, z, start_offset, required_sectors)?;
            } else {
                // we do not fit in the current gap, need to find a new home for
                // this chunk.
                self.offsets.remove(i); // this chunk will no longer be here.
                let offset = *self.offsets.last().unwrap() as u64;

                // add a new offset representing the new 'end' of the current region file.
                self.offsets.push(offset + required_sectors as u64);
                self.set_chunk(offset, scheme, chunk)?;
                self.set_header(x, z, offset, required_sectors)?;
            }
        }

        Ok(())
    }
}

// impl<S: Seek + Read + Send + Sync> Region for RegionBuffer<S> {
//     fn chunk(&mut self, x: CCoord, z: CCoord) -> Option<JavaChunk> {
//         let loc = self.chunk_location(x.0 as usize, z.0 as usize).ok()?;

//         let data = self.load_chunk(loc.x, loc.z).ok()?;

//         let res = JavaChunk::from_bytes(&data);

//         match &res {
//             Ok(_) => {}
//             Err(e) => println!("{}", e),
//         }

//         res.ok()
//     }
// }

/// The location of chunk data within a Region file.
#[derive(Debug, PartialEq)]
pub struct ChunkLocation {
    pub begin_sector: usize,
    pub sector_count: usize,
    pub x: usize,
    pub z: usize,
}

/// Various compression schemes that NBT data is typically compressed with.
#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum CompressionScheme {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
}

/// Encodes how the NBT-Data is compressed
#[derive(Debug)]
pub struct ChunkMeta {
    pub compressed_len: u32,
    pub compression_scheme: CompressionScheme,
}

impl ChunkMeta {
    pub fn new(data: &[u8]) -> Result<Self> {
        if data.len() < 5 {
            return Err(Error::InsufficientData);
        }

        let mut buf = &data[..5];
        let len = buf.read_u32::<BigEndian>()?;
        let scheme = buf.read_u8()?;
        let scheme = CompressionScheme::try_from(scheme).map_err(|_| Error::InvalidChunkMeta)?;

        Ok(Self {
            compressed_len: len - 1, // this len include the compression byte.
            compression_scheme: scheme,
        })
    }
}

impl<S: Seek + Read + Write> RegionBuffer<S> {
    pub fn new(data: S) -> Result<Self> {
        let mut tmp = Self {
            data,
            offsets: vec![],
        };

        let mut max_offset = 0;
        let mut max_offsets_sector_count = 0;

        for z in 0..32 {
            for x in 0..32 {
                let (off, count) = tmp.info(x, z)?;
                if off == 0 && count == 0 {
                    continue;
                }

                tmp.offsets.push(off);
                if off > max_offset {
                    max_offset = off;
                    max_offsets_sector_count = count;
                }
            }
        }

        tmp.offsets.sort_unstable();
        tmp.offsets.push(max_offset + max_offsets_sector_count);
        Ok(tmp)
    }

    /// Return the (region-relative) Chunk location (x, z)
    pub fn chunk_location(&mut self, x: usize, z: usize) -> Result<ChunkLocation> {
        if x >= 32 || z >= 32 {
            return Err(Error::InvalidOffset(x as isize, z as isize));
        }

        let pos = 4 * ((x % 32) + (z % 32) * 32);

        self.data.seek(SeekFrom::Start(pos as u64))?;

        let mut buf = [0u8; 4];
        self.data.read_exact(&mut buf[..])?;

        let mut off = 0usize;
        off |= (buf[0] as usize) << 16;
        off |= (buf[1] as usize) << 8;
        off |= buf[2] as usize;
        let count = buf[3] as usize;
        Ok(ChunkLocation {
            begin_sector: off,
            sector_count: count,
            x,
            z,
        })
    }

    /// Return the raw, uncompressed NBT data for a chunk at the
    /// (region-relative) Chunk location (x, z). Region's hold 32 by 32 chunks.
    ///
    /// Can be further processed with [`stream::Parser`] or even with
    /// `Blob::from_reader()` of hematite_nbt.
    ///
    /// [`stream::Parser`]: ../stream/struct.Parser.html
    pub fn load_chunk(&mut self, x: usize, z: usize) -> Result<Vec<u8>> {
        let data = self.load_raw_chunk_at(x, z)?;
        decompress_chunk(&data)
    }

    /// Call function with each uncompressed, non-empty chunk, calls f(x, z, data).
    pub fn for_each_chunk(&mut self, mut f: impl FnMut(usize, usize, &Vec<u8>)) -> Result<()> {
        let mut offsets = Vec::<ChunkLocation>::new();

        // Build list of existing chunks
        for x in 0..32 {
            for z in 0..32 {
                let loc = self.chunk_location(x, z)?;
                // 0,0 chunk location means the chunk isn't present.
                // cannot decide if this means we should return an error from chunk_location() or not.
                if loc.begin_sector != 0 && loc.sector_count != 0 {
                    offsets.push(loc);
                }
            }
        }

        // sort so we linearly seek through the file.
        // might make things easier on a HDD [citation needed]
        offsets.sort_by(|o1, o2| o2.begin_sector.cmp(&o1.begin_sector));

        for offset in offsets {
            let chunk = self.load_chunk(offset.x, offset.z)?;
            f(offset.x, offset.z, &chunk);
        }

        Ok(())
    }

    /// Return the raw, compressed data for a chunk at ChunkLocation
    fn load_raw_chunk(&mut self, offset: &ChunkLocation, dest: &mut Vec<u8>) -> Result<()> {
        self.data.seek(SeekFrom::Start(
            offset.begin_sector as u64 * SECTOR_SIZE as u64,
        ))?;

        dest.resize(5, 0);
        self.data.read_exact(&mut dest[0..5])?;
        let metadata = ChunkMeta::new(&dest[..5])?;

        dest.resize(5 + metadata.compressed_len as usize, 0u8);

        self.data.read_exact(&mut dest[5..])?;
        Ok(())
    }

    /// Return the raw, compressed data for a chunk at the (region-relative) Chunk location (x, z)
    fn load_raw_chunk_at(&mut self, x: usize, z: usize) -> Result<Vec<u8>> {
        let location = self.chunk_location(x, z)?;

        // 0,0 chunk location means the chunk isn't present.
        if location.begin_sector != 0 && location.sector_count != 0 {
            let mut buf = Vec::new();
            self.load_raw_chunk(&location, &mut buf)?;
            Ok(buf)
        } else {
            Err(Error::ChunkNotFound)
        }
    }
}

// Read Information Bytes of Minecraft Chunk and decompress it
fn decompress_chunk(data: &[u8]) -> Result<Vec<u8>> {
    // Metadata encodes the length in bytes and the compression type
    let meta = ChunkMeta::new(data).unwrap();

    // compressed data starts at byte 5
    let inbuf = &mut &data[5..];
    let mut decoder = match meta.compression_scheme {
        CompressionScheme::Zlib => ZlibDecoder::new(inbuf),
        _ => panic!("unknown compression scheme (gzip?)"),
    };
    let mut outbuf = Vec::new();
    // read the whole Chunk
    decoder.read_to_end(&mut outbuf)?;
    Ok(outbuf)
}

// copied from rust std unstable_div_ceil function
pub const fn unstable_div_ceil(lhs: usize, rhs: usize) -> usize {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 && rhs > 0 {
        d + 1
    } else {
        d
    }
}
