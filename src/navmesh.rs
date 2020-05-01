use crate::Rect;
use aabb_quadtree::Spatial;
use bitbuffer::{BitRead, BitReadStream, Endianness, ReadError};
use euclid::{TypedPoint2D, TypedSize2D};
use std::fmt;
use std::fmt::Debug;
use std::ops::Index;

/// A 3 dimensional coordinate
#[derive(Debug, BitRead)]
pub struct Vector3(pub f32, pub f32, pub f32);

/// A unique identifier for a navigation area
#[derive(Debug, BitRead, Clone, Copy, Eq, PartialEq)]
pub struct NavAreaId(u32);

impl fmt::Display for NavAreaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A navigation area from the nav file
#[derive(Debug)]
pub struct NavArea {
    pub id: NavAreaId,
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

impl NavArea {
    pub fn width(&self) -> f32 {
        self.south_east.0 - self.north_west.0
    }
    pub fn height(&self) -> f32 {
        self.south_east.1 - self.north_west.1
    }

    /// Get the z height of a x/y point inside the navigation area
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sourcenav::get_area_tree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///   let file = std::fs::read("path/to/navfile.nav")?;
    ///   let tree = get_area_tree(file)?;
    ///   let area = tree.query(150.0, -312.0).next().unwrap();
    ///   
    ///   let height = area.get_z_height(150.0, -312.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_z_height(&self, x: f32, y: f32) -> f32 {
        let from_east = self.south_east.0 - x;
        let from_south = self.south_east.0 - y;

        let north_slope = (self.north_west.2 - self.north_east_z) / self.width();
        let south_slope = (self.south_west_z - self.south_east.2) / self.width();

        let north_z = self.north_east_z + north_slope * from_east;
        let south_z = self.south_east.2 + south_slope * from_east;

        let final_slope = (north_z - south_z) / self.height();

        south_z + final_slope * from_south
    }
}

pub(crate) struct HammerUnit;

impl Spatial<HammerUnit> for NavArea {
    fn aabb(&self) -> Rect {
        Rect {
            origin: TypedPoint2D::new(self.north_west.0, self.north_west.1),
            size: TypedSize2D::new(
                self.south_east.0 - self.north_west.0,
                self.south_east.1 - self.north_west.1,
            ),
        }
    }
}

/// The connections from a navigation area into it's neighbours
///
/// Contains a list of area id's for every [`NavDirection`]
///
/// # Examples
///
/// ```no_run
/// # fn get_connections_from_somewhere() -> sourcenav::Connections {
/// #    Default::default()
/// # }
/// use sourcenav::NavDirection;
///
/// let connections = get_connections_from_somewhere();
///
/// let north_connections = &connections[NavDirection::North];
/// ```
///
/// [`NavDirection`]: ./enum.NavDirection.html
#[derive(Debug, Default)]
pub struct Connections([Vec<NavAreaId>; 4]);

impl<E: Endianness> BitRead<E> for Connections {
    fn read(stream: &mut BitReadStream<E>) -> Result<Self, ReadError> {
        let mut connections = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        for direction in connections.iter_mut() {
            let connection_count: u32 = stream.read()?;
            direction.reserve(connection_count as usize);
            for _ in 0..connection_count {
                direction.push(stream.read()?);
            }
        }

        Ok(Connections(connections))
    }
}

impl Index<NavDirection> for Connections {
    type Output = Vec<NavAreaId>;

    fn index(&self, index: NavDirection) -> &Self::Output {
        &self.0[index as u8 as usize]
    }
}

/// The connections from a navigation area into it's neighbours
///
/// Contains a list of area id's for every [`NavDirection`]
///
/// # Examples
///
/// ```no_run
/// # fn get_ladder_connections_from_somewhere() -> sourcenav::LadderConnections {
/// #    Default::default()
/// # }
/// use sourcenav::LadderDirection;
///
/// let connections = get_ladder_connections_from_somewhere();
///
/// let down_connections = &connections[LadderDirection::Down];
/// ```
///
/// [`NavDirection`]: ./enum.NavDirection.html
#[derive(Debug, Default)]
pub struct LadderConnections([Vec<NavAreaId>; 2]);

impl<E: Endianness> BitRead<E> for LadderConnections {
    fn read(stream: &mut BitReadStream<E>) -> Result<Self, ReadError> {
        let mut connections = [Vec::new(), Vec::new()];

        for direction in connections.iter_mut() {
            let connection_count: u32 = stream.read()?;
            direction.reserve(connection_count as usize);
            for _ in 0..connection_count {
                direction.push(stream.read()?);
            }
        }

        Ok(LadderConnections(connections))
    }
}

impl Index<LadderDirection> for LadderConnections {
    type Output = Vec<NavAreaId>;

    fn index(&self, index: LadderDirection) -> &Self::Output {
        &self.0[index as u8 as usize]
    }
}

/// The directions in which two areas can be connected
#[derive(Debug, BitRead)]
#[repr(u8)]
#[discriminant_bits = 8]
pub enum NavDirection {
    North,
    East,
    South,
    West,
}

/// The directions in which two areas can be connected by ladder
#[derive(Debug, BitRead)]
#[repr(u8)]
#[discriminant_bits = 8]
pub enum LadderDirection {
    Up,
    Down,
}

/// A hiding spot within an area
#[derive(Debug, BitRead)]
pub struct NavHidingSpot {
    id: u32,
    location: Vector3,
    flags: u8,
}

/// An area that can be used for approach, no longer used in newer nav files
#[derive(Debug, BitRead)]
pub struct ApproachArea {
    approach_here: u32,
    approach_pre: u32,
    approach_type: u8,
    approach_next: u32,
    approach_how: u8,
}

/// A path that can be used to approach an area
#[derive(Debug, BitRead)]
pub struct EncounterPath {
    from_area_id: NavAreaId,
    from_direction: u8,
    to_area_id: NavAreaId,
    to_direction: u8,
    #[size_bits = 8]
    spots: Vec<EncounterSpot>,
}

#[derive(Debug, BitRead)]
pub struct EncounterSpot {
    order: u32,
    distance: u8, // divide by 255
}

/// The light intensity at the four corners of an area
#[derive(Debug, BitRead, Default)]
pub struct LightIntensity {
    pub north_west: f32,
    pub north_east: f32,
    pub south_west: f32,
    pub south_east: f32,
}

/// An area that is visible
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
