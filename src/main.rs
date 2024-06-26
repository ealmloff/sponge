use std::{
    f32::consts::{FRAC_PI_2, TAU},
    time::Duration,
};

use async_timer::Interval;
use dioxus:: prelude::*; 
use nalgebra::Vector2;

fn main() {
    launch(app);
}

fn app() -> Element {
    let count = 40;
    rsx! {
        svg {
            width: "1000px",
            height: "1000px",

            for i in 0..count {
                Blob {
                    count: i*10,
                    color: "hsla({(i * 360)/count},80%,50%,0.05)",
                    stroke: "rgba(0,0,0,0.01)",
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct BlobProps {
    count: usize,
    color: String,
    stroke: String,
}

fn Blob(props: BlobProps) -> Element {
    let count = props.count;
    let mut obj = use_signal( || Object {
        points: (0..count)
            .map(|i| {
                let f = (i as f32 * TAU - FRAC_PI_2) / count as f32;
                Vector2::new(f.cos(), f.sin()) * 100.0
            })
            .map(|pos| Point {
                loctation: pos + Vector2::new(500.0, 500.0),
                velocity: -pos,
            })
            .collect(),
    });

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let duration = Duration::from_millis(10);
        let mut interval = Interval::platform_new(duration);
        async move {
            loop {
                interval.wait().await;
                let midpoint = obj
                    .read()
                    .points
                    .iter()
                    .map(|point| point.loctation)
                    .sum::<Vector2<f32>>()
                    / obj.read().points.len() as f32;
                obj.write().update(duration.as_millis() as f32 / 1000.0);
                let points = &mut obj.write().points;
                for i in 0..points.len() {
                    let point = points[i];
                    for i2 in 0..i {
                        let current = &mut points[i2];
                        let diff = point.loctation - current.loctation;
                        let inv = diff.normalize() / diff.magnitude();
                        current.velocity -= inv;
                        points[i].velocity += inv;
                    }
                    // sufface tension
                    let len = points.len();
                    let next_idx = (i + 1) % len;
                    let next = points[next_idx];
                    let diff = point.loctation - next.loctation;
                    let ideal = diff.normalize() * 0.5;
                    let delta = ideal - diff;
                    points[i].velocity += delta;
                    points[next_idx].velocity -= delta;
                }
                for point in points {
                    point.velocity += (midpoint - point.loctation).normalize() * 1.0;
                    point.velocity +=
                        (Vector2::new(500.0, 500.0) - point.loctation).normalize() * 0.25;
                }
            }
        }
    });

    let path = obj.read().to_string();
    rsx! {
        path {
            d: "{path}",
            stroke: "{props.stroke}",
            fill: "{props.color}",
        }
    }
}

struct Object {
    points: Vec<Point>,
}

impl Object {
    fn to_string(&self) -> String {
        let mut s = String::new();
        let mut iterator = self.points.iter();
        if let Some(first) = iterator.next() {
            s.push_str(&"M ");
            s.push_str(&point_to_string(*first));
            for p in iterator {
                s.push_str(" ");
                s.push_str(&point_to_string(*p));
            }
        }
        if let Some(first) = self.points.first() {
            s.push_str(" ");
            s.push_str(&point_to_string(*first));
        }
        s
    }

    fn update(&mut self, dt: f32) {
        for p in &mut self.points {
            p.update(dt);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Point {
    loctation: Vector2<f32>,
    velocity: Vector2<f32>,
}

impl Point {
    fn update(&mut self, dt: f32) {
        self.loctation += self.velocity * dt;
    }
}

fn point_to_string(point: Point) -> String {
    format!("{} {}", point.loctation.x, point.loctation.y)
}
