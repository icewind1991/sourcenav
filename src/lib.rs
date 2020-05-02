use crate::navmesh::HammerUnit;
pub use crate::navmesh::{
    ApproachArea, Connections, EncounterPath, LadderConnections, LadderDirection, LightIntensity,
    NavDirection, NavHidingSpot, Vector3, VisibleArea,
};
use crate::parser::read_areas;
pub use crate::parser::{NavArea, ParseError};
use aabb_quadtree::{ItemId, QuadTree};
use bitbuffer::{BitReadStream, LittleEndian};
use euclid::{TypedPoint2D, TypedRect, TypedSize2D};

mod navmesh;
mod parser;

type Rect = TypedRect<f32, HammerUnit>;

/// A tree of all navigation areas
pub struct NavTree(QuadTree<NavArea, HammerUnit, [(ItemId, Rect); 4]>);

/// Parse all navigation areas from a nav file
///
/// ## Examples
///
/// ```no_run
/// use sourcenav::get_area_tree;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = std::fs::read("path/to/navfile.nav")?;
/// let tree = get_area_tree(file)?;
/// # Ok(())
/// # }
/// ```
pub fn get_area_tree(data: impl Into<BitReadStream<LittleEndian>>) -> Result<NavTree, ParseError> {
    let areas = read_areas(data.into())?;

    let (min_x, min_y, max_x, max_y) = areas.iter().fold(
        (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
        |(min_x, min_y, max_x, max_y), area| {
            (
                f32::min(min_x, area.north_west.0),
                f32::min(min_y, area.north_west.1),
                f32::max(max_x, area.south_east.0),
                f32::max(max_y, area.south_east.1),
            )
        },
    );

    let mut tree = QuadTree::default(
        Rect::new(
            TypedPoint2D::new(min_x - 1.0, min_y - 1.0),
            TypedSize2D::new(max_x - min_x + 2.0, max_y - min_y + 2.0),
        ),
        areas.len(),
    );

    for area in areas {
        tree.insert(area);
    }

    Ok(NavTree(tree))
}

impl NavTree {
    /// Find the navigation areas at a x/y cooordinate
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use sourcenav::get_area_tree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = std::fs::read("path/to/navfile.nav")?;
    /// let tree = get_area_tree(file)?;
    /// let areas = tree.query(150.0, -312.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn query(&self, x: f32, y: f32) -> impl Iterator<Item = &NavArea> {
        let query_box = Rect::new(TypedPoint2D::new(x, y), TypedSize2D::new(1.0, 1.0));

        self.0.query(query_box).into_iter().map(|(area, ..)| area)
    }

    /// Find the z-height of a specfic x/y cooordinate
    ///
    /// Note that multiple heights might exist for a given x/y coooridnate
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use sourcenav::get_area_tree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = std::fs::read("path/to/navfile.nav")?;
    /// let tree = get_area_tree(file)?;
    /// let heights = tree.find_z_height(150.0, -312.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_z_height<'a>(&'a self, x: f32, y: f32) -> impl Iterator<Item = f32> + 'a {
        self.query(x, y).map(move |area| area.get_z_height(x, y))
    }

    /// Get the z height at a point
    ///
    /// A z-guess should be provided to resolve cases where multiple z values are possible
    pub fn find_best_height(&self, x: f32, y: f32, z_guess: f32) -> f32 {
        let found_heights = self.find_z_height(x, y);
        let best_z = f32::MIN;

        found_heights.fold(best_z, |best_z, found_z| {
            if (found_z - z_guess).abs() < (best_z - z_guess).abs() {
                found_z
            } else {
                best_z
            }
        })
    }

    /// Get all navigation areas from the nav file
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use sourcenav::get_area_tree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = std::fs::read("path/to/navfile.nav")?;
    /// let tree = get_area_tree(file)?;
    /// for area in tree.areas() {
    ///     println!("area: {}", area.id)
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn areas(&self) -> impl Iterator<Item = &NavArea> {
        self.0.iter().map(|(_, (area, _))| area)
    }
}

#[test]
fn test_tree() {
    let file = std::fs::read("data/pl_badwater.nav").unwrap();
    let tree = get_area_tree(file).unwrap();

    // single flat plane
    let point1 = (1600.0, -1300.0);

    assert_eq!(
        vec![375.21506],
        tree.find_z_height(point1.0, point1.1).collect::<Vec<f32>>()
    );

    // 2 z levels
    let point2 = (360.0, -1200.0);

    assert_eq!(
        vec![290.2907, 108.144775],
        tree.find_z_height(point2.0, point2.1).collect::<Vec<f32>>()
    );

    // top of slope
    let point3 = (320.0, -1030.0);

    assert_eq!(
        vec![220.83125],
        tree.find_z_height(point3.0, point3.1).collect::<Vec<f32>>()
    );

    // bottom of same slope
    let point4 = (205.0, -1030.0);

    assert_eq!(
        vec![147.23126],
        tree.find_z_height(point4.0, point4.1).collect::<Vec<f32>>()
    );

    assert_eq!(
        tree.query(point3.0, point3.1).next().map(|area| area.id),
        tree.query(point4.0, point4.1).next().map(|area| area.id)
    );
}

#[cfg(doctest)]
doc_comment::doctest!("../README.md");
