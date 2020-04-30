use bitbuffer::{BitRead, BitReadStream, Endianness, ReadError};
use std::ops::Index;

#[derive(Debug, BitRead)]
pub struct Vector3(pub f32, pub f32, pub f32);

#[derive(Debug)]
pub struct NavArea {
    pub id: u32,
    pub north_west: Vector3,
    pub south_east: Vector3,
    pub north_east_z: f32,
    pub south_west_z: f32,
    pub flags: u32,
    pub connections: Connections,
    pub hiding_spots: Vec<NavHidingSpot>,
    pub approach_areas: Vec<ApproachArea>,
    pub encounter_paths: Vec<EncounterPath>,
    pub place: u16,
    pub light_intensity: LightIntensity,
    pub ladder_connections: LadderConnections,
    pub earliest_occupy_first_team: f32,
    pub earliest_occupy_second_team: f32,
    pub visible_areas: Vec<VisibleArea>,
    pub inherit_visibility_from_area_id: u32,
}

#[derive(Debug)]
pub struct Connections([Vec<u32>; 4]);

impl<E: Endianness> BitRead<E> for Connections {
    fn read(stream: &mut BitReadStream<E>) -> Result<Self, ReadError> {
        let mut connections = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        for direction in 0..4 {
            let connection_count: u32 = stream.read()?;
            connections[direction] = Vec::with_capacity(connection_count as usize);
            for _ in 0..connection_count {
                connections[direction].push(stream.read()?);
            }
        }

        Ok(Connections(connections))
    }
}

impl Index<NavDirection> for Connections {
    type Output = Vec<u32>;

    fn index(&self, index: NavDirection) -> &Self::Output {
        &self.0[index as u8 as usize]
    }
}

#[derive(Debug)]
pub struct LadderConnections([Vec<u32>; 2]);

impl<E: Endianness> BitRead<E> for LadderConnections {
    fn read(stream: &mut BitReadStream<E>) -> Result<Self, ReadError> {
        let mut connections = [Vec::new(), Vec::new()];

        for direction in 0..2 {
            let connection_count: u32 = stream.read()?;
            connections[direction] = Vec::with_capacity(connection_count as usize);
            for _ in 0..connection_count {
                connections[direction].push(stream.read()?);
            }
        }

        Ok(LadderConnections(connections))
    }
}

impl Index<LadderDirection> for LadderConnections {
    type Output = Vec<u32>;

    fn index(&self, index: LadderDirection) -> &Self::Output {
        &self.0[index as u8 as usize]
    }
}

#[derive(Debug, BitRead)]
#[repr(u8)]
#[discriminant_bits = 8]
pub enum NavDirection {
    North,
    East,
    South,
    West,
}

#[derive(Debug, BitRead)]
#[repr(u8)]
#[discriminant_bits = 8]
pub enum LadderDirection {
    Up,
    Down,
}

#[derive(Debug, BitRead)]
pub struct NavHidingSpot {
    id: u32,
    location: Vector3,
    flags: u8,
}

#[derive(Debug, BitRead)]
pub struct ApproachArea {
    approach_here: u32,
    approach_pre: u32,
    approach_type: u8,
    approach_next: u32,
    approach_how: u8,
}

#[derive(Debug, BitRead)]
pub struct EncounterPath {
    from_area_id: u32,
    from_direction: u8,
    to_area_id: u32,
    to_direction: u8,
    #[size_bits = 8]
    spots: Vec<EncounterSpot>,
}

#[derive(Debug, BitRead)]
pub struct EncounterSpot {
    order: u32,
    distance: u8, // divide by 255
}

#[derive(Debug, BitRead, Default)]
pub struct LightIntensity {
    pub north_west: f32,
    pub north_east: f32,
    pub south_west: f32,
    pub south_east: f32,
}

#[derive(Debug, BitRead)]
pub struct VisibleArea {
    id: u32,
    attributes: u8,
}

#[derive(Debug)]
pub struct NavPlace {
    id: u32,
    name: String,
}

#[derive(Debug)]
pub struct NavMesh {}
