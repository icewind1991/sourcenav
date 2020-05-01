pub use crate::navmesh::NavArea;
use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
use err_derive::Error;

/// Errors that can occur when parsing the binary nav file
#[derive(Debug, Error)]
pub enum ParseError {
    /// An error ocured when reading from the source binary data
    #[error(display = "Error while reading from data: {}", _0)]
    ReadError(#[error(source)] bitbuffer::ReadError),
    #[error(
        display = "Invalid magic number ({:#8X}), not a nav file or corrupted",
        _0
    )]
    /// The binary data contained an invalid magic number and is probably not a nav file
    InvalidMagicNumber(u32),
    /// The version of the nav file is not supported by this parser
    #[error(display = "The major version for this nav ({}), is not supported", _0)]
    UnsupportedVersion(u32),
}

pub(crate) fn read_areas(data: Vec<u8>) -> Result<Vec<NavArea>, ParseError> {
    let mut data = BitReadStream::new(BitReadBuffer::new(data, LittleEndian));
    let magic = data.read()?;
    if magic != 0xFEED_FACE {
        return Err(ParseError::InvalidMagicNumber(magic));
    }

    let major_version: u32 = data.read()?;

    if major_version < 6 || major_version > 16 {
        return Err(ParseError::UnsupportedVersion(major_version));
    }

    let _minor_version: u32 = if major_version >= 10 { data.read()? } else { 0 };

    let _size: u32 = data.read()?;

    let _is_analysed = if major_version >= 14 {
        data.read_int::<u8>(8)? == 1
    } else {
        false
    };

    let place_count: u16 = data.read()?;

    // let places = Vec::with_capacity(place_count as usize);
    for _id in 1..=place_count {
        let name_length: u16 = data.read()?;
        let _name = data.read_string(Some(name_length as usize))?;
        // TODO
    }

    let _has_unnamed_areas = if major_version >= 12 {
        data.read_int::<u8>(8)? == 1
    } else {
        false
    };

    let area_count: u32 = data.read()?;

    let mut areas = Vec::with_capacity(area_count as usize);

    for _ in 0..area_count {
        let id = data.read()?;

        let flags = if major_version <= 8 {
            data.read_int(8)?
        } else if major_version <= 12 {
            data.read_int(16)?
        } else {
            data.read_int(32)?
        };

        let north_west = data.read()?;
        let south_east = data.read()?;
        let north_east_z = data.read()?;
        let south_west_z = data.read()?;

        let connections = data.read()?;

        let hiding_spots_count: u8 = data.read()?;
        let hiding_spots = data.read_sized(hiding_spots_count as usize)?;

        let approach_areas = if major_version < 15 {
            let approach_area_count: u8 = data.read()?;

            data.read_sized(approach_area_count as usize)?
        } else {
            Vec::new()
        };

        let encounter_paths_count: u32 = data.read()?;
        let encounter_paths = data.read_sized(encounter_paths_count as usize)?;

        let place = data.read()?;

        let ladder_connections = data.read()?;

        let earliest_occupy_first_team = data.read()?;
        let earliest_occupy_second_team = data.read()?;

        let light_intensity = if major_version >= 11 {
            data.read()?
        } else {
            Default::default()
        };

        let visible_areas = if major_version >= 16 {
            let visible_areas_count: u32 = data.read()?;
            data.read_sized(visible_areas_count as usize)?
        } else {
            Vec::new()
        };

        let inherit_visibility_from_area_id = data.read()?;

        data.skip_bits(32)?;

        areas.push(NavArea {
            id,
            north_west,
            south_east,
            north_east_z,
            south_west_z,
            flags,
            connections,
            hiding_spots,
            approach_areas,
            encounter_paths,
            place,
            ladder_connections,
            earliest_occupy_first_team,
            earliest_occupy_second_team,
            light_intensity,
            visible_areas,
            inherit_visibility_from_area_id,
        });
    }

    debug_assert!(data.bits_left() <= 32);

    Ok(areas)
}

#[test]
fn test() {
    let file = std::fs::read("data/pl_badwater.nav").unwrap();
    let areas = read_areas(file).unwrap();
    assert_eq!(1930, areas.len());
}
