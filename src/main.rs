use std::mem::Discriminant;

use cgmath::{EuclideanSpace, InnerSpace, MetricSpace, Point3, Vector3};
use color_eyre::{
    eyre::{Context, Report},
    owo_colors::OwoColorize,
};
use image::{ImageBuffer, Rgba};

struct Sphere {
    center: Point3<f32>,
    radius: f32,
}

struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    fn hit(&self, sphere: &Sphere) -> bool {
        let origin_vec = self.origin.to_vec();
        let sphere_center_vec = sphere.center.to_vec();

        // Quadratic equation in the form: a*t^2 + 2k*t + c = 0
        let a = self.direction.dot(self.direction);
        let k = origin_vec.dot(self.direction) - self.direction.dot(sphere_center_vec);
        let c = origin_vec.dot(origin_vec) - 2.0 * origin_vec.dot(sphere_center_vec)
            + sphere_center_vec.dot(sphere_center_vec)
            - sphere.radius * sphere.radius;

        let discriminant_squared = k * k - a * c;

        return discriminant_squared >= 0.0;
    }
}

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let (width, height) = (854, 480);

    let mut image = ImageBuffer::new(width, height);

    let sphere = Sphere {
        center: Point3::new(0.0, 0.0, 0.0),
        radius: 1.0,
    };

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let ray = Ray {
            origin: Point3::new(
                x as f32 / width as f32 * 2.0 - 1.0,
                y as f32 / height as f32 * -2.0 + 1.0,
                -1.0,
            ),
            direction: Vector3::new(0.0, 0.0, 1.0),
        };

        if ray.hit(&sphere) {
            *pixel = Rgba([255u8, 0, 255, 255]);
        } else {
            *pixel = Rgba([135u8, 206, 235, 255]);
        }
    }

    image.save("output.png").wrap_err("Cannot save image")?;

    Ok(())
}
