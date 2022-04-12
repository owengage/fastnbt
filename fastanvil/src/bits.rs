//! functionality around bit manipulations specific to the Anvil file format.

use bit_field::{BitArray, BitField};
use fastnbt::LongArray;
use serde::Deserialize;

// Various data versions for the anvil format
const V1_17_0: i32 = 2724;
const V1_17_1: i32 = 2730;
const SNAPSHOT_21W44A: i32 = 2845;

/// PackedBits can be used in place of blockstates in chunks to avoid
/// allocating memory for them when they might not be needed. This object by
/// default just retains a reference to the data in the input, and `unpack_into`
/// can be used to get the unpacked version when needed.
#[derive(Deserialize, Debug)]
pub struct PackedBits(pub LongArray);

impl PackedBits {
    pub fn unpack_blockstates(&self, bits_per_item: usize, buf: &mut [u16]) {
        let bpi = match self.0.len() {
            256 => 4,
            342 => 5,
            410 => 6,
            456 => 7,
            512 => 8,
            586 => 9,
            _ => return self.unpack_1_15(bits_per_item, buf),
        };

        self.unpack_1_16(bpi, buf)
    }

    fn unpack_1_16(&self, bits_per_item: usize, buf: &mut [u16]) {
        let data = &*self.0;

        let mut buf_i = 0;
        let values_per_64bits = 64 / bits_per_item;

        for datum in data {
            let datum = *datum as u64;
            for i in 0..values_per_64bits {
                let v = datum.get_bits(i * bits_per_item..(i + 1) * bits_per_item);
                buf[buf_i] = v as u16;
                buf_i += 1;
                if buf_i >= buf.len() {
                    break;
                }
            }
        }
    }

    fn unpack_1_15(&self, bits_per_item: usize, buf: &mut [u16]) {
        // TODO: Get around having to allocate here.
        let mut v: Vec<u64> = vec![];
        for datum in &*self.0 {
            v.push(*datum as u64);
        }

        for (i, val) in buf.iter_mut().enumerate() {
            let begin = i * bits_per_item;
            let end = begin + bits_per_item;
            *val = v.get_bits(begin..end) as u16;
        }
    }
}

/// Expand blockstate data so each block is an element of a `Vec`.
///
/// This requires the number of items in the palette of the section the blockstates came from. This is because
/// blockstate is packed with on a bit-level granularity. If the maximum index in the palette fits in 5 bits, then
/// every 5 bits of the blockstates will represent a block.
////
/// In 1.15 there is no padding, so blocks bleed into one another, so remainder bits are tracked and handled for you.
/// In 1.16 padding bits are used so that a block is always in a single 64-bit int.
pub fn expand_blockstates(data: &[i64], palette_len: usize) -> Vec<u16> {
    let bits_per_item = bits_per_block(palette_len);
    let blocks_per_section = 16 * 16 * 16;

    // If it's tightly packed assume 1.15 format.
    if blocks_per_section * bits_per_item == data.len() * 64 {
        expand_generic_1_15(data, bits_per_item)
    } else {
        expand_generic_1_16(data, bits_per_item)
    }
}

/// Expand heightmap data. This is equivalent to `expand_generic(data, 9)`.
pub fn expand_heightmap(data: &[i64], y_min: isize, data_version: i32) -> Vec<i16> {
    let bits_per_item = 9;

    let _after1_17 = data_version >= 2695;
    const LEN_1_16_TO_17: usize = 37;
    const LEN_1_15: usize = 36;

    // Likely that 1.18 will have something like this to calculate world
    // heights. It was how the snapshots did it.

    // // We extract 256 9-bit **unsigned** integers from the data. This is
    // // one integer per column in the chunk. In 1.16 onwards we store 7
    // // of these per 64bit long, meaning there's padding bits, and a long
    // // at the end with extra values that are unused. We resize down to
    // // get rid of these extra values.
    // let mut v = expand_generic_1_16(data, bits_per_item);
    // v.resize(256, 0);

    // // Switch to signed. If 1.17 subtract 64 to allow the negative heights.
    // let shift = if after1_17 { -64 } else { 0 };
    // v.into_iter().map(|h| h as i16 + shift).collect()

    match data_version {
        V1_17_0 | V1_17_1 | SNAPSHOT_21W44A.. => {
            let bits_per = match data.len() {
                43 => 10,
                37 => 9,
                len => panic!("Len of heightmap data: {}", len),
            };

            let mut v = expand_generic_1_16(data, bits_per);
            v.resize(256, 0);

            // Reinterpret as signed and offset by min y.
            v.into_iter().map(|h| (h as isize + y_min) as i16).collect()
        }
        _ => match data.len() {
            LEN_1_16_TO_17 => {
                // We extract 256 9-bit **unsigned** integers from the data. This is
                // one integer per column in the chunk. In 1.16 onwards we store 7
                // of these per 64bit long, meaning there's padding bits, and a long
                // at the end with extra values that are unused. We resize down to
                // get rid of these extra values.
                let mut v = expand_generic_1_16(data, bits_per_item);
                v.resize(256, 0);

                // Reinterpret as signed.
                v.into_iter().map(|h| h as i16).collect()
            }
            LEN_1_15 => expand_generic_1_15(data, bits_per_item)
                .into_iter()
                .map(|h| h as i16)
                .collect(),
            _ => panic!("did not understand height format, try calculated mode"),
        },
    }
}

/// Expand generic bit-packed data in the 1.16 format, ie with padding bits.
pub fn expand_generic_1_16(data: &[i64], bits: usize) -> Vec<u16> {
    let values_per_64bits = 64 / bits;
    let mut result: Vec<u16> = Vec::with_capacity(values_per_64bits * data.len());

    for datum in data {
        for i in 0..values_per_64bits {
            let datum = *datum as u64;
            let v = datum.get_bits(i * bits..(i + 1) * bits);
            result.push(v as u16);
        }
    }

    result
}

/// Expand generic bit-packed data in the 1.15 format, ie data potentially existing across two 64-bit ints.
pub fn expand_generic_1_15(data: &[i64], bits: usize) -> Vec<u16> {
    let mut result: Vec<u16> = vec![0; (data.len() * 64) / bits];

    // Unfortunely make a copy here in order to treat the data as u64 rather than i64.
    // At some point we will change the parser to let us take the data as u64 rather than i64.
    let copy: Vec<_> = data.iter().map(|i| *i as u64).collect();

    for (i, val) in result.iter_mut().enumerate() {
        let begin = i * bits;
        let end = begin + bits;
        *val = copy.get_bits(begin..end) as u16;
    }

    result
}

/// Get the number of bits that will be used in `Blockstates` per block.
///
/// See `anvil::expand_blockstates` for more information.
pub fn bits_per_block(palette_len: usize) -> usize {
    if palette_len < 16 {
        4
    } else {
        std::cmp::max((palette_len as f64).log2().ceil() as usize, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BigEndian, ReadBytesExt};

    #[test]
    fn nether_heightmap_v1_15_2() {
        let input: Vec<i64> = vec![
            2310355422147575936,
            1155177711073787968,
            577588855536893984,
            288794427768446992,
            144397213884223496,
            72198606942111748,
            36099303471055874,
            -9205322385119247871,
            4620710844295151872,
            2310355422147575936,
            1155177711073787968,
            577588855536893984,
            288794427768446992,
            144397213884223496,
            72198606942111748,
            36099303471055874,
            -9205322385119247871,
            4620710844295151872,
            2310355422147575936,
            1155177711073787968,
            577588855536893984,
            288794427768446992,
            144397213884223496,
            72198606942111748,
            36099303471055874,
            -9205322385119247871,
            4620710844295151872,
            2310355422147575936,
            1155177711073787968,
            577588855536893984,
            288794427768446992,
            144397213884223496,
            72198606942111748,
            36099303471055874,
            -9205322385119247871,
            4620710844295151872,
        ];

        let actual = expand_heightmap(&input[..], 0, 0);
        assert_eq!(&[128; 16 * 16][..], actual.as_slice());
    }

    #[test]
    fn heightmap_overworld_v1_15_2() {
        let input: Vec<i64> = vec![
            1299610109330100808,
            649787462479005732,
            329397330866873490,
            -9060925171218247159,
            4692909455540619556,
            2346453626107004050,
            -8050144124289646015,
            5198158688002654496,
            2599149849916022926,
            -7941846763497811896,
            649769835865982755,
            -1985452877601561582,
            8230641191400739272,
            4692909451237263588,
            2057661397361812594,
            -7906029485971705287,
            5126101092889936160,
            2599079343463931022,
            -7941846763497811896,
            -3970923381816146141,
            -6606172535203224687,
            8230641191400739144,
            2960142884643391716,
            2057660297841779794,
            -8483335214034816455,
            5126100816936184084,
            2526951243511307406,
            -7941882016858338234,
            -8591634191684319453,
            -4295817113055649007,
            7075463488933695880,
            3537731740163475652,
            1768865870081737826,
            -8338939101813906895,
            5053902485947822360,
            2526951242973911180,
        ];

        let actual = expand_heightmap(&input[..], 0, 0);
        assert_eq!(
            &[
                72, 73, 72, 72, 72, 73, 72, 72, 72, 72, 72, 72, 72, 72, 72, 73, 72, 72, 72, 72, 73,
                72, 72, 72, 73, 72, 72, 72, 72, 73, 72, 73, 73, 72, 72, 72, 73, 72, 72, 72, 71, 72,
                72, 72, 72, 72, 72, 73, 72, 72, 72, 72, 72, 73, 71, 71, 72, 71, 72, 72, 72, 72, 72,
                72, 72, 72, 72, 72, 71, 71, 71, 71, 71, 71, 71, 71, 71, 72, 72, 72, 72, 72, 72, 72,
                71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 72, 72, 72, 72, 72, 72, 71, 71, 71, 71, 72,
                71, 71, 71, 71, 71, 72, 72, 72, 73, 72, 72, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71,
                71, 72, 72, 72, 72, 72, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 69, 69, 69,
                69, 69, 69, 69, 71, 71, 71, 71, 71, 71, 71, 71, 71, 69, 69, 69, 69, 69, 69, 69, 71,
                71, 71, 71, 71, 71, 71, 71, 71, 69, 69, 69, 69, 69, 69, 70, 71, 71, 71, 71, 71, 72,
                70, 70, 70, 70, 70, 70, 70, 70, 70, 71, 71, 71, 71, 71, 71, 70, 70, 70, 70, 70, 70,
                70, 70, 70, 70, 70, 71, 71, 71, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70,
                70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70,
                70, 70, 70, 70
            ][..],
            &actual[..]
        );
    }

    #[test]
    fn palette_size_checks() {
        assert_eq!(4, bits_per_block(2));

        assert_eq!(4, bits_per_block(3));
        assert_eq!(4, bits_per_block(4));

        assert_eq!(4, bits_per_block(5));
        assert_eq!(4, bits_per_block(7));
        assert_eq!(4, bits_per_block(8));

        assert_eq!(4, bits_per_block(9));
        assert_eq!(4, bits_per_block(16));
        assert_eq!(10, bits_per_block(1 << 10));
    }

    #[test]
    fn unpack_1_15_heightmap() {
        let height_data = vec![
            16u8, 8, 4, 2, 1, 0, 128, 64, 200, 68, 18, 9, 4, 130, 64, 32, 227, 241, 249, 8, 126,
            65, 34, 16, 18, 1, 0, 128, 64, 32, 16, 7, 248, 252, 126, 63, 33, 144, 136, 68, 0, 128,
            64, 31, 143, 199, 227, 241, 126, 63, 33, 16, 136, 36, 2, 1, 63, 31, 143, 199, 227, 241,
            248, 252, 32, 144, 72, 36, 2, 1, 0, 128, 143, 199, 227, 241, 248, 252, 126, 63, 8, 4,
            2, 1, 0, 126, 63, 31, 227, 241, 248, 252, 126, 63, 32, 144, 2, 1, 0, 126, 63, 31, 143,
            199, 248, 252, 126, 63, 32, 16, 8, 4, 252, 126, 63, 31, 143, 199, 227, 241, 126, 63,
            32, 16, 8, 4, 2, 0, 63, 31, 143, 199, 227, 241, 248, 252, 32, 16, 8, 4, 2, 0, 252, 126,
            143, 199, 227, 241, 248, 252, 126, 63, 8, 4, 1, 248, 252, 126, 63, 31, 227, 241, 248,
            252, 126, 63, 32, 16, 1, 248, 252, 126, 63, 31, 143, 199, 248, 252, 126, 63, 32, 16, 8,
            4, 252, 126, 63, 31, 143, 199, 227, 241, 126, 63, 32, 16, 8, 3, 241, 248, 63, 31, 143,
            199, 227, 241, 248, 252, 32, 16, 8, 3, 241, 248, 252, 126, 143, 199, 227, 241, 248,
            252, 126, 63, 135, 227, 241, 248, 252, 126, 63, 31, 227, 241, 248, 252, 126, 63, 30,
            143, 241, 248, 252, 126, 63, 31, 143, 199, 248, 252, 126, 63, 30, 143, 71, 227, 252,
            126, 63, 31, 143, 199, 227, 241, 126, 63, 30, 143, 71, 227, 241, 248, 63, 31, 143, 199,
            227, 241, 248, 252, 30, 143, 71, 227, 241, 248, 252, 126,
        ];

        let mut height_ints = vec![];

        let data = &mut height_data.as_slice();

        while let Ok(datum) = data.read_i64::<BigEndian>() {
            height_ints.push(datum);
        }

        let expected = [
            64, 64, 64, 64, 64, 64, 64, 64, 64, 65, 65, 65, 65, 66, 67, 68, 65, 63, 66, 63, 63, 63,
            64, 64, 64, 64, 64, 64, 65, 66, 66, 67, 63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64, 64,
            64, 65, 66, 66, 63, 63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64, 64, 65, 65, 65, 63, 63,
            63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64, 64, 64, 65, 63, 63, 63, 63, 63, 63, 63, 63,
            63, 63, 64, 64, 64, 64, 64, 64, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64,
            64, 64, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64, 64, 64, 63, 63, 63, 63,
            63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64, 64, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
            63, 63, 64, 64, 64, 64, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64,
            63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 64, 64, 64, 63, 63, 63, 63, 63, 63,
            63, 63, 63, 63, 63, 63, 63, 63, 62, 61, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
            63, 63, 61, 61, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 61, 61, 63, 63,
            63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 61, 61,
        ];

        let packed = PackedBits(LongArray::new(height_ints));
        let mut buf = vec![0; 16 * 16];
        packed.unpack_blockstates(9, buf.as_mut_slice());
        assert_eq!(&expected[..], &buf[..]);
    }
}
