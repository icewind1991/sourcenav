# SourceNav

parsing of SourceEngine `.nav` files

## Usage

This library is currently focused on getting the z-height from an x/y coordinate in a map and the api is tailored towards
that usage. For other usages the raw navigation areas are exposed.

```rust
use sourcenav::get_area_tree;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::read("data/pl_badwater.nav")?;
    let tree = get_area_tree(file)?;

    assert_eq!(220.83125,  tree.find_best_height(320.0, -1030.0, 0.0));

    Ok(())
}

```