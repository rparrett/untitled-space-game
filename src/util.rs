use bevy::{
    math::Vec2,
    prelude::Mesh,
    render::mesh::{Indices, PrimitiveTopology},
};

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

pub fn point_in_rect(point: Vec2, min: Vec2, max: Vec2) -> bool {
    point.x < min.x || point.x > max.x || point.y < min.y || point.y > max.y
}

pub fn chevron(width: f32, height: f32, thickness: f32) -> Mesh {
    let half_width = width / 2.;
    let half_height = height / 2.;

    let positions = vec![
        [0., -half_height, 0.],
        [half_width, half_height - thickness, 0.],
        [half_width, half_height, 0.],
        [0., -half_height + thickness, 0.],
        [-half_width, half_height, 0.],
        [-half_width, half_height - thickness, 0.],
    ];

    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];

    let mut uvs = Vec::with_capacity(positions.len());

    for pos in positions.iter() {
        uvs.push([
            (pos[0] + half_width / width),
            (pos[0] + half_height / height),
        ])
    }

    let indices = vec![
        1, 3, 0, //
        2, 3, 1, //
        3, 1, 2, //
        5, 0, 3, //
        4, 5, 3, //
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}
