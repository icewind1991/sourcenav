use crate::navmesh::HammerUnit;
use crate::parser::read_areas;
pub use crate::parser::{NavArea, ParseError};
use aabb_quadtree::{ItemId, QuadTree};
use euclid::{TypedPoint2D, TypedRect, TypedSize2D};

mod navmesh;
mod parser;

pub struct NavTree(QuadTree<NavArea, HammerUnit, [(ItemId, TypedRect<f32, HammerUnit>); 4]>);

pub fn get_area_tree(data: Vec<u8>) -> Result<NavTree, ParseError> {
    let areas = read_areas(data)?;

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
        TypedRect::new(
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
    pub fn query(
        &self,
        x: f32,
        y: f32,
    ) -> impl Iterator<Item = (&NavArea, TypedRect<f32, HammerUnit>, ItemId)> {
        let query_box = TypedRect::new(TypedPoint2D::new(x, y), TypedSize2D::new(1.0, 1.0));

        self.0.query(query_box).into_iter()
    }

    pub fn find_z_height<'a>(&'a self, x: f32, y: f32) -> impl Iterator<Item = f32> + 'a {
        self.query(x, y)
            .map(move |(area, ..)| area.get_z_height(x, y))
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
        tree.query(point3.0, point3.1)
            .next()
            .map(|(area, ..)| area.id),
        tree.query(point4.0, point4.1)
            .next()
            .map(|(area, ..)| area.id)
    );
}
