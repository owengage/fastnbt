use flate2::read::ZlibDecoder;
use std::convert::TryFrom;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;

use crate::{CCoord, JavaChunk};
use crate::{Error, Result};

/// the size in bytes of a 'sector' in a region file. Sectors are Minecraft's size unit
/// for chunks. For example, a chunk might be `3 * SECTOR_SIZE` bytes.
pub const SECTOR_SIZE: usize = 4096;

/// the size of the region file header.
pub const HEADER_SIZE: usize = 2 * SECTOR_SIZE;

pub trait Region: Send + Sync {
    /// Load the chunk at the given chunk coordinates, ie 0..32 for x and z.
    /// Implmentations do not need to be concerned with caching chunks they have
    /// loaded, this will be handled by the types using the region.
    fn chunk(&self, x: CCoord, z: CCoord) -> Option<JavaChunk>;
}

pub trait RegionRead {
    fn read_chunk(&self, x: CCoord, z: CCoord) -> Result<Vec<u8>>;
}

pub trait RegionWrite {
    fn write_chunk(&self, x: CCoord, z: CCoord, chunk: &[u8]) -> Result<()>;
}

/// A Minecraft Region.
pub struct RegionBuffer<S> {
    data: Mutex<S>,
}

impl<S> RegionBuffer<S>
where
    S: Write + Seek,
{
    pub fn new_empty(mut buf: S) -> Result<Self> {
        buf.seek(SeekFrom::Start(0))?;

        buf.write_all(&[0; HEADER_SIZE])?;

        Ok(Self {
            data: Mutex::new(buf),
        })
    }
}

impl<S> RegionRead for RegionBuffer<S>
where
    S: Seek + Read,
{
    fn read_chunk(&self, x: CCoord, z: CCoord) -> Result<Vec<u8>> {
        if x.0 >= 32 || z.0 >= 32 {
            return Err(Error::InvalidOffset(x.0, z.0));
        }

        let pos = 4 * ((x.0 % 32) + (z.0 % 32) * 32);

        let mut lock = self.data.lock().unwrap();
        lock.seek(SeekFrom::Start(pos as u64))?;

        let mut buf = [0u8; 4];
        lock.read_exact(&mut buf[..])?;

        drop(lock);

        let mut off = 0usize;
        off |= (buf[0] as usize) << 16;
        off |= (buf[1] as usize) << 8;
        off |= buf[2] as usize;
        let count = buf[3] as usize;

        if off == 0 && count == 0 {
            Err(Error::ChunkNotFound)
        } else {
            // Ok(ChunkLocation {
            //     begin_sector: off,
            //     sector_count: count,
            //     x,
            //     z,
            // });

            todo!()
        }
    }
}

impl<S> RegionWrite for RegionBuffer<S>
where
    S: Seek + Write,
{
    fn write_chunk(&self, x: CCoord, z: CCoord, chunk: &[u8]) -> Result<()> {
        // compress the bytes
        // does it fit in the existing hole?
        // we need to track where the next chunk in memory is...
        todo!()
    }
}

impl<S: Seek + Read + Send + Sync> Region for RegionBuffer<S> {
    fn chunk(&self, x: CCoord, z: CCoord) -> Option<JavaChunk> {
        let loc = self.chunk_location(x.0 as usize, z.0 as usize).ok()?;

        let data = self.load_chunk(loc.x, loc.z).ok()?;

        let res = JavaChunk::from_bytes(&data);

        match &res {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

        res.ok()
    }
}

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

impl<S: Seek + Read> RegionBuffer<S> {
    pub fn new(data: S) -> Self {
        Self {
            data: Mutex::new(data),
        }
    }

    /// Return the (region-relative) Chunk location (x, z)
    pub fn chunk_location(&self, x: usize, z: usize) -> Result<ChunkLocation> {
        if x >= 32 || z >= 32 {
            return Err(Error::InvalidOffset(x as isize, z as isize));
        }

        let pos = 4 * ((x % 32) + (z % 32) * 32);

        let mut lock = self.data.lock().unwrap();
        lock.seek(SeekFrom::Start(pos as u64))?;

        let mut buf = [0u8; 4];
        lock.read_exact(&mut buf[..])?;

        drop(lock);

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
    pub fn load_chunk(&self, x: usize, z: usize) -> Result<Vec<u8>> {
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
    fn load_raw_chunk(&self, offset: &ChunkLocation, dest: &mut Vec<u8>) -> Result<()> {
        let mut lock = self.data.lock().unwrap();
        lock.seek(SeekFrom::Start(
            offset.begin_sector as u64 * SECTOR_SIZE as u64,
        ))?;

        dest.resize(5, 0);
        lock.read_exact(&mut dest[0..5])?;
        let metadata = ChunkMeta::new(&dest[..5])?;

        dest.resize(5 + metadata.compressed_len as usize, 0u8);

        lock.read_exact(&mut dest[5..])?;
        Ok(())
    }

    /// Return the raw, compressed data for a chunk at the (region-relative) Chunk location (x, z)
    fn load_raw_chunk_at(&self, x: usize, z: usize) -> Result<Vec<u8>> {
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
