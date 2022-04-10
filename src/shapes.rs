use crate::vector::Vec3;

pub enum Shape {
    Rectangle(Rectangle),
    Circle(Circle),
}

pub trait Collides {
    fn intersects(&self, shape: &Shape) -> bool;
}

pub struct Circle {
    pub center: Vec3,
    pub radius: f32,
}

pub struct Rectangle {
    pub top_left: Vec3,
    pub width: f32,
    pub height: f32,
}

impl Collides for Shape {
    fn intersects(&self, shape: &Shape) -> bool {
        match self {
            Shape::Rectangle(this) => match shape {
                Shape::Rectangle(other) => collides_rectangle_rectangle(this, other),
                Shape::Circle(other) => collides_circle_rectangle(other, this),
            },
            Shape::Circle(this) => match shape {
                Shape::Rectangle(other) => collides_circle_rectangle(this, other),
                Shape::Circle(other) => collides_circle_circle(this, other),
            },
        }
    }
}

/*
          |------s-----|--r-|
          |--------d--------|
        *  *           t----------
     *     r1 *        |          |
    *    c1----*      p|----c2    | h/2
    *          *       |          |
     *        *         ----------
        *  *                w/2
*/

fn collides_circle_rectangle(circle: &Circle, rectangle: &Rectangle) -> bool {
    let half_size = Vec3::new(rectangle.width / 2.0, rectangle.height / 2.0, 0.0);
    let rect_center = rectangle.top_left + half_size;

    let d = circle.center - rect_center;
    let clamped = Vec3::new(
        d.x().clamp(-half_size.x(), half_size.x()),
        d.y().clamp(-half_size.y(), half_size.y()),
        0.0,
    );

    let p = rect_center + clamped;
    let s = p - circle.center;

    s.length() <= circle.radius
}

/*
          |--------d-------|
        *  *              *  *
     *      r1*        *r2      *
    *    c1----*      *----c2    *
    *          *      *          *
     *        *        *        *
        *  *              *  *

*/
fn collides_circle_circle(a: &Circle, b: &Circle) -> bool {
    let d = a.center - b.center;
    d.length() <= a.radius + b.radius
}

/*
    a---------
    |         |
    |    b----|---
     ----|----    |
         |        |
          --------
*/
fn collides_rectangle_rectangle(a: &Rectangle, b: &Rectangle) -> bool {
    // x-axis
    a.top_left.x() + a.width >= b.top_left.x() &&
    b.top_left.x() + b.width >= a.top_left.x() &&
    // y-axis
    a.top_left.y() + a.height >= b.top_left.y() &&
    b.top_left.y() + b.height >= a.top_left.y()
}
