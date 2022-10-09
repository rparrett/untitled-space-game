use bevy::{
    math::Vec2,
    prelude::Mesh,
    render::mesh::{Indices, PrimitiveTopology},
};
use rand::{thread_rng, Rng};

pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

struct Intercept {
    pos: f32,
    edge: Edge,
}

/// Given a direction vector and the minimum and maximum bounds of a rectangle,
/// project a ray from 0, 0 and return the intersection with that rectangle.
/// https://math.stackexchange.com/questions/2738250/intersection-of-ray-starting-inside-square-with-that-square
pub fn project_onto_bounding_rectangle(dir: Vec2, min: Vec2, max: Vec2) -> Option<(Vec2, Edge)> {
    let maybe_x_int: Option<Intercept> = if dir.x > 0. {
        Some(Intercept {
            pos: max.x / dir.x,
            edge: Edge::Right,
        })
    } else if dir.x < 0. {
        Some(Intercept {
            pos: min.x / dir.x,
            edge: Edge::Left,
        })
    } else {
        None
    };

    let maybe_y_int: Option<Intercept> = if dir.y > 0. {
        Some(Intercept {
            pos: max.y / dir.y,
            edge: Edge::Top,
        })
    } else if dir.y < 0. {
        Some(Intercept {
            pos: min.y / dir.y,
            edge: Edge::Bottom,
        })
    } else {
        None
    };

    match (maybe_x_int, maybe_y_int) {
        (None, Some(y_int)) => Some((Vec2::new(y_int.pos * dir.x, y_int.pos * dir.y), y_int.edge)),
        (Some(x_int), None) => Some((Vec2::new(x_int.pos * dir.x, x_int.pos * dir.y), x_int.edge)),
        (Some(x_int), Some(y_int)) => {
            if x_int.pos < y_int.pos {
                Some((Vec2::new(x_int.pos * dir.x, x_int.pos * dir.y), x_int.edge))
            } else if y_int.pos < x_int.pos {
                Some((Vec2::new(y_int.pos * dir.x, y_int.pos * dir.y), y_int.edge))
            } else {
                Some((Vec2::new(x_int.pos * dir.x, x_int.pos * dir.y), x_int.edge))
            }
        }
        (None, None) => None,
    }
}

pub fn point_in_rect(point: Vec2, min: Vec2, max: Vec2) -> bool {
    !(point.x < min.x || point.x > max.x || point.y < min.y || point.y > max.y)
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

/// Generate `num` u32s with the sum of `total` and minimum value `min`
pub fn random_u32_subdivisions(num: u32, total: u32, min: u32) -> Vec<u32> {
    debug_assert!(total > num * min);

    let mut rng = thread_rng();

    let mut vals = Vec::with_capacity(num as usize);
    let mut sum = 0;

    for i in 0..(num - 1) {
        let max = total - sum - min * (num - i - 1);
        let val = rng.gen_range(min..max);
        vals.push(val);
        sum += val;
    }

    vals.push(total - sum);

    vals
}

/// Generate an ordered set of `num` f32s between 0.0 and `max`
///
/// The absolute difference between values is at least `min_gap`.
///
/// That minimum gap also "wraps" around from the last value to the first.

pub fn random_circular_f32_distribution(num: u32, min_gap: f32, max: f32) -> Vec<f32> {
    debug_assert!(num as f32 * min_gap < max);

    let mut rng = thread_rng();

    let mut vals = Vec::with_capacity(num as usize);
    let mut last = 0.0;

    for i in 0..num {
        let start = if i == 0 { last } else { last + min_gap };
        let gap_allowance = (num - i - 1) as f32 * min_gap;
        let end = max - gap_allowance;

        last = rng.gen_range(start..end);

        vals.push(last);
    }

    vals
}
