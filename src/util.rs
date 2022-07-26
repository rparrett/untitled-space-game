use bevy::math::Vec2;

/// Given a direction vector and the minimum and maximum bounds of a rectangle,
/// project a ray from 0, 0 and return the intersection with that rectangle.
pub fn project_onto_bounding_rectangle(dir: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    let x_intercept: Option<f32> = if dir.x > 0. {
        Some(max.x / dir.x)
    } else if dir.x < 0. {
        Some(min.x / dir.x)
    } else {
        None
    };

    let y_intercept: Option<f32> = if dir.y > 0. {
        Some(max.y / dir.y)
    } else if dir.y < 0. {
        Some(min.y / dir.y)
    } else {
        None
    };

    match (x_intercept, y_intercept) {
        (None, Some(ty)) => Vec2::new(ty * dir.x, ty * dir.y),
        (Some(tx), None) => Vec2::new(tx * dir.x, tx * dir.y),
        (Some(tx), Some(ty)) => {
            if tx < ty {
                Vec2::new(tx * dir.x, tx * dir.y)
            } else if ty < tx {
                Vec2::new(ty * dir.x, ty * dir.y)
            } else {
                Vec2::new(tx * dir.x, tx * dir.y)
            }
        }
        (None, None) => {
            panic!()
        }
    }
}
