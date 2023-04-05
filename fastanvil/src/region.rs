use std::convert::TryFrom;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::ZlibEncoder;
use flate2::Compression;
use num_enum::TryFromPrimitive;

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

/// A Minecraft Region, allowing reading and writing of chunk data to a stream (eg a
/// File). This does not concern itself with manipulating chunk data, users are
/// expected to use `fastnbt` or other deserialization method to manipulate the
/// chunk data itself.
#[derive(Clone)]
pub struct Region<S> {
    stream: S,
    // last offset is always the next valid place to write a chunk.
    offsets: Vec<u64>,
}

impl<S> Region<S>
where
    S: Read + Seek,
{
    /// Load a region from an existing stream. Will assume a seek of zero is the
    /// start of the region. This does not load all region data into memory.
    /// Chunks are read from the underlying stream when needed.
    pub fn from_stream(stream: S) -> Result<Self> {
        // Could delay the offsets loading until a write_chunk occurs. It's not
        // needed when only reading. But rendering some worlds with and without
        // the calculation doesn't really show much perf benefit.
        let mut tmp = Self {
            stream,
            offsets: vec![],
        };

        let mut max_offset = 0;
        let mut max_offsets_sector_count = 0;

        for z in 0..32 {
            for x in 0..32 {
                let loc = tmp.location(x, z)?;
                if loc.offset == 0 && loc.sectors == 0 {
                    continue;
                }

                tmp.offsets.push(loc.offset);
                if loc.offset > max_offset {
                    max_offset = loc.offset;
                    max_offsets_sector_count = loc.sectors;
                }
            }
        }

        tmp.offsets.sort_unstable();

        // we add an offset representing the end of sectors that are in use.
        tmp.offsets.push(max_offset + max_offsets_sector_count);
        Ok(tmp)
    }

    /// Read the chunk located at the chunk coordindates `x`, `z`. The chunk
    /// data returned is uncompressed NBT. `Ok(None)` means that the chunk does
    /// not exist, which will be the case if that chunk has not generated.  If
    /// `x` or `z` are outside `0..32`, [`Error::InvalidOffset`] is returned.
    pub fn read_chunk(&mut self, x: usize, z: usize) -> Result<Option<Vec<u8>>> {
        self.compression_scheme(x, z)?
            .map(|scheme| match scheme {
                CompressionScheme::Zlib => {
                    let mut decoder = flate2::write::ZlibDecoder::new(vec![]);
                    self.read_compressed_chunk(x, z, &mut decoder)?;
                    Ok(decoder.finish()?)
                }
                CompressionScheme::Gzip => {
                    let mut decoder = flate2::write::GzDecoder::new(vec![]);
                    self.read_compressed_chunk(x, z, &mut decoder)?;
                    Ok(decoder.finish()?)
                }
                CompressionScheme::Uncompressed => {
                    let mut buf = vec![];
                    self.read_compressed_chunk(x, z, &mut buf)?;
                    Ok(buf)
                }
            })
            .transpose()
    }

    /// Get the location of the chunk in the stream.
    pub(crate) fn location(&mut self, x: usize, z: usize) -> Result<ChunkLocation> {
        if x >= 32 || z >= 32 {
            return Err(Error::InvalidOffset(x as isize, z as isize));
        }

        self.stream.seek(SeekFrom::Start(header_pos(x, z)))?;

        let mut buf = [0u8; 4];
        self.stream.read_exact(&mut buf[..])?;

        let mut offset = 0u64;
        offset |= (buf[0] as u64) << 16;
        offset |= (buf[1] as u64) << 8;
        offset |= buf[2] as u64;
        let sectors = buf[3] as u64;

        Ok(ChunkLocation { offset, sectors })
    }

    /// Low level method. Read a compressed chunk into the given writer. The
    /// `compression_scheme` method can be used to discover how the chunk
    /// written is compressed, allowing you to write directly to a decompresser.
    ///
    /// Returns a bool indicating if a chunk was found at the given x,z.
    fn read_compressed_chunk(
        &mut self,
        x: usize,
        z: usize,
        writer: &mut dyn Write,
    ) -> Result<bool> {
        let loc = self.location(x, z)?;

        if loc.offset == 0 && loc.sectors == 0 {
            Ok(false)
        } else {
            self.stream
                .seek(SeekFrom::Start(loc.offset * SECTOR_SIZE as u64))?;

            let mut buf = [0u8; 5];
            self.stream.read_exact(&mut buf)?;
            let metadata = ChunkMeta::new(&buf)?;

            let mut adapted = (&mut self.stream).take(metadata.compressed_len as u64);

            io::copy(&mut adapted, writer)?;

            Ok(true)
        }
    }

    /// Return the inner buffer used. The buffer is rewound to the logical end of the stream,
    /// meaning the position at which the chunk data of the region ends. If the region does not
    /// contain any chunk the position is at the end of the header.
    ///
    /// # Examples
    /// This can be used to truncate a region file after manipulating it to save disk space.
    /// ```no_run
    /// # use fastanvil::Region;
    /// # use fastanvil::Result;
    /// # use std::io::Seek;
    /// # fn main() -> Result<()> {
    /// let mut file = std::fs::File::open("foo.mca")?;
    /// let mut region = Region::from_stream(file)?;
    /// // manipulate region
    /// let mut file = region.into_inner()?;
    /// let len = file.stream_position()?;
    /// file.set_len(len)?;
    /// # Ok(())
    /// # }
    ///  ```
    pub fn into_inner(mut self) -> io::Result<S> {
        // pop the last element so we can access the second last. This can be unwrapped safely as
        // there should always be at least one offset
        self.offsets.pop().unwrap();
        let Some(offset) = self.offsets.pop() else {
            self.stream.seek(SeekFrom::Start(REGION_HEADER_SIZE as u64))?;
            return Ok(self.stream)
        };
        self.stream
            .seek(SeekFrom::Start(offset * SECTOR_SIZE as u64))?;
        // add 4 to the chunk length in order to compensate for the 4 bytes the length field takes
        // up itself
        let chunk_length = self.stream.read_u32::<BigEndian>()? + 4;
        let logical_end = unstable_div_ceil(
            offset as usize * SECTOR_SIZE + chunk_length as usize,
            SECTOR_SIZE,
        ) * SECTOR_SIZE;

        self.stream.seek(SeekFrom::Start(logical_end as u64))?;
        Ok(self.stream)
    }

    /// Low level method. Get the compression scheme that a given chunk is
    /// compressed with in the region. Used in conjuction with
    /// `read_compressed_chunk`.
    fn compression_scheme(&mut self, x: usize, z: usize) -> Result<Option<CompressionScheme>> {
        if x >= 32 || z >= 32 {
            return Err(Error::InvalidOffset(x as isize, z as isize));
        }

        let loc = self.location(x, z)?;

        if loc.offset == 0 && loc.sectors == 0 {
            Ok(None)
        } else {
            self.stream
                .seek(SeekFrom::Start(loc.offset * SECTOR_SIZE as u64))?;

            let mut buf = [0u8; 5];
            self.stream.read_exact(&mut buf)?;
            let metadata = ChunkMeta::new(&buf)?;

            Ok(Some(metadata.compression_scheme))
        }
    }

    pub fn iter(&mut self) -> RegionIter<'_, S> {
        RegionIter::new(self)
    }

    fn chunk_meta(&self, compressed_chunk_size: u32, scheme: CompressionScheme) -> [u8; 5] {
        let mut buf = [0u8; 5];
        let mut c = Cursor::new(buf.as_mut_slice());

        // size written to disk includes the byte representing the compression
        // scheme, so +1.
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

impl<S> Region<S>
where
    S: Read + Write + Seek,
{
    /// Create an new empty region. The provided stream will be overwritten, and
    /// will assume a seek to 0 is the start of the region. The stream needs
    /// read, write, and seek like a file provides.
    pub fn new(mut stream: S) -> Result<Self> {
        stream.rewind()?;
        stream.write_all(&[0; REGION_HEADER_SIZE])?;

        Ok(Self {
            stream,
            offsets: vec![2], // 2 is the end of the header
        })
    }

    /// Write the given uncompressed NBT chunk data to the chunk coordinates x,
    /// z. The chunk data will be compressed with zlib by default. You can use
    /// write_compressed_chunk if you want more control. If `x` or `z` are
    /// outside `0..32`, [`Error::InvalidOffset`] is returned.
    pub fn write_chunk(&mut self, x: usize, z: usize, uncompressed_chunk: &[u8]) -> Result<()> {
        let mut buf = vec![];
        let mut enc = ZlibEncoder::new(uncompressed_chunk, Compression::fast());
        enc.read_to_end(&mut buf)?;
        self.write_compressed_chunk(x, z, CompressionScheme::Zlib, &buf)
    }

    /// Low level method. Write the given compressed chunk data to the stream.
    /// It is the callers responsibility to make sure the compression scheme
    /// matches the compression used. If `x` or `z` are outside `0..32`,
    /// [`Error::InvalidOffset`] is returned.
    pub fn write_compressed_chunk(
        &mut self,
        x: usize,
        z: usize,
        scheme: CompressionScheme,
        compressed_chunk: &[u8],
    ) -> Result<()> {
        let loc = self.location(x, z)?;
        let required_sectors =
            unstable_div_ceil(CHUNK_HEADER_SIZE + compressed_chunk.len(), SECTOR_SIZE);

        if loc.offset == 0 && loc.sectors == 0 {
            // chunk does not exist in the region yet.
            let offset = *self.offsets.last().expect("offset should always exist");

            // add a new offset representing the new 'end' of the current region file.
            self.offsets.push(offset + required_sectors as u64);
            self.set_chunk(offset, scheme, compressed_chunk)?;
            self.pad()?;
            self.set_header(x, z, offset, required_sectors)?;
        } else {
            // chunk already exists in the region file, need to update it.
            let i = self.offsets.binary_search(&loc.offset).unwrap();
            let start_offset = self.offsets[i];
            let end_offset = self.offsets[i + 1];
            let available_sectors = (end_offset - start_offset) as usize;

            if required_sectors <= available_sectors {
                // we fit in the current gap in the file.
                self.set_chunk(start_offset, scheme, compressed_chunk)?;
                self.set_header(x, z, start_offset, required_sectors)?;
            } else {
                // we do not fit in the current gap, need to find a new home for
                // this chunk.
                self.offsets.remove(i); // this chunk will no longer be here.
                let offset = *self.offsets.last().unwrap();

                // add a new offset representing the new 'end' of the current region file.
                self.offsets.push(offset + required_sectors as u64);
                self.set_chunk(offset, scheme, compressed_chunk)?;
                self.pad()?;
                self.set_header(x, z, offset, required_sectors)?;
            }
        }

        Ok(())
    }

    /// Remove the chunk at the chunk location with the coordinates x and z. Frees up space if
    /// possible.
    pub fn remove_chunk(&mut self, x: usize, z: usize) -> Result<()> {
        let loc = self.location(x, z)?;
        // zero the region header for the chunk
        self.set_header(x, z, 0, 0)?;

        // remove the offset of the chunk
        let i = self.offsets.binary_search(&loc.offset).unwrap();
        self.offsets.remove(i);

        Ok(())
    }

    /// Write the chunk data to the given offset, does no checking.
    fn set_chunk(&mut self, offset: u64, scheme: CompressionScheme, chunk: &[u8]) -> Result<()> {
        self.stream
            .seek(SeekFrom::Start(offset * SECTOR_SIZE as u64))?;

        self.stream.write_all(&self.chunk_meta(
            chunk.len() as u32, // doesn't include header size
            scheme,
        ))?;

        self.stream.write_all(chunk)?;
        Ok(())
    }

    fn pad(&mut self) -> Result<()> {
        let current_end = unstable_stream_len(&mut self.stream)? as usize;
        let padded_end = unstable_div_ceil(current_end, SECTOR_SIZE) * SECTOR_SIZE;
        let pad_len = padded_end - current_end;
        self.stream.write(&vec![0; pad_len])?;
        Ok(())
    }

    /// Write to the header for the given chunk.
    fn set_header(
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
        buf[3] = new_sector_count as u8;

        // seek to header
        self.stream.seek(SeekFrom::Start(header_pos(x, z)))?;
        self.stream.write_all(&buf)?;
        Ok(())
    }
}

/// Various compression schemes that NBT data is typically compressed with.
#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum CompressionScheme {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
}

pub struct RegionIter<'a, S>
where
    S: Read + Seek,
{
    inner: &'a mut Region<S>,
    index: usize,
}

impl<'a, S> RegionIter<'a, S>
where
    S: Read + Seek,
{
    fn new(inner: &'a mut Region<S>) -> Self {
        Self { inner, index: 0 }
    }

    fn next_xz(&mut self) -> Option<(usize, usize)> {
        let index = self.index;
        self.index += 1;

        if index == 32 * 32 {
            return None;
        }

        let x = index % 32;
        let z = index / 32;
        Some((x, z))
    }
}
pub struct ChunkData {
    pub x: usize,
    pub z: usize,
    pub data: Vec<u8>,
}

impl<'a, S> Iterator for RegionIter<'a, S>
where
    S: Read + Seek,
{
    type Item = Result<ChunkData>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((x, z)) = self.next_xz() {
            let c = self.inner.read_chunk(x, z);

            match c {
                Ok(Some(c)) => return Some(Ok(ChunkData { x, z, data: c })),
                Ok(None) => {} // chunk absent, fine.
                Err(e) => return Some(Err(e)),
            }
        }

        None
    }
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

fn unstable_stream_len(seek: &mut impl Seek) -> Result<u64> {
    let old_pos = seek.stream_position()?;
    let len = seek.seek(SeekFrom::End(0))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        seek.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len)
}

fn header_pos(x: usize, z: usize) -> u64 {
    (4 * ((x % 32) + (z % 32) * 32)) as u64
}

#[derive(Debug)]
pub struct ChunkLocation {
    /// The offset, in units of 4kiB sectors, into the region file this chunk is
    /// located at. Offset 0 is the start of the file.
    pub offset: u64,

    /// The number of 4 kiB sectors that this chunk occupies in the region file.
    pub sectors: u64,
}

/// Encodes how the NBT-Data is compressed
#[derive(Debug)]
struct ChunkMeta {
    pub compressed_len: u32,
    pub compression_scheme: CompressionScheme,
}

impl ChunkMeta {
    fn new(mut data: &[u8]) -> Result<Self> {
        let len = data.read_u32::<BigEndian>()?;
        let scheme = data.read_u8()?;
        let scheme =
            CompressionScheme::try_from(scheme).map_err(|_| Error::UnknownCompression(scheme))?;

        Ok(Self {
            compressed_len: len - 1, // this len include the compression byte.
            compression_scheme: scheme,
        })
    }
}
