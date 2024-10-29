use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective { fov: f32 },
    Orthographic { size: f32 },
}

pub struct Camera {
    position: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,

    projection: Projection,
    near_clip: f32,
    far_clip: f32,

    view_matrix: Matrix4<f32>,
    proj_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        near_clip: f32,
        far_clip: f32,
        projection: Projection,
    ) -> Self {
        let mut camera = Self {
            position,
            target,
            up,
            near_clip,
            far_clip,
            projection,
            view_matrix: Matrix4::<_>::identity(),
            proj_matrix: Matrix4::<_>::identity(),
        };
        camera.update_matrices();

        camera
    }

    pub fn position(&self) -> Point3<f32> {
        self.position
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
        self.update_matrices();
    }

    pub fn target(&self) -> Point3<f32> {
        self.target
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
        self.update_matrices();
    }

    pub fn up(&self) -> Vector3<f32> {
        self.up
    }

    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up;
        self.update_matrices();
    }

    pub fn near_clip(&self) -> f32 {
        self.near_clip
    }

    pub fn set_near_clip(&mut self, near_clip: f32) {
        self.near_clip = near_clip;
        self.update_matrices();
    }

    pub fn far_clip(&self) -> f32 {
        self.far_clip
    }

    pub fn set_far_clip(&mut self, far_clip: f32) {
        self.far_clip = far_clip;
        self.update_matrices();
    }

    pub fn projection(&self) -> Projection {
        self.projection
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.update_matrices();
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    pub fn proj_matrix(&self) -> Matrix4<f32> {
        self.proj_matrix
    }

    fn update_matrices(&mut self) {
        todo!()
    }
}
