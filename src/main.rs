use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3, Vector4, VectorSpace};
use color_eyre::eyre::{Context, Report};
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
        let aspect_ratio = width as f32 / height as f32;
        let ray = Ray {
            origin: Point3::new(
                x as f32 / width as f32 * aspect_ratio * 2.0 - aspect_ratio,
                y as f32 / height as f32 * -2.0 + 1.0,
                -1.0,
            ),
            direction: Vector3::new(0.0, 0.0, 1.0),
        };

        let color = if ray.hit(&sphere) {
            Vector4::new(1.0, 0.0, 1.0, 1.0)
        } else {
            let up = Vector3::unit_y();
            let cosine_similarity =
                ray.direction.dot(up) / (ray.direction.magnitude() * up.magnitude());

            let top_color = Vector4::new(0.53, 0.8, 0.92, 1.0);
            let bottom_color = Vector4::new(1.0, 1.0, 1.0, 1.0);

            top_color.lerp(bottom_color, (cosine_similarity + 1.0) / 2.0)
        };

        *pixel = Rgba([
            (color.x * 255.0) as u8,
            (color.y * 255.0) as u8,
            (color.z * 255.0) as u8,
            (color.w * 255.0) as u8,
        ]);
    }

    image.save("output.png").wrap_err("Cannot save image")?;

    Ok(())
}
