use cgmath::{
    Deg, Matrix4, MetricSpace, Point3, SquareMatrix, Transform, Vector2, Vector3, Vector4,
};
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective { fov: Deg<f32> },
    Orthographic { size: f32 },
}

pub struct Camera {
    position: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,

    resolution_x: u32,
    resolution_y: u32,

    projection: Projection,
    near_clip: f32,
    far_clip: f32,

    view_matrix: Matrix4<f32>,
    proj_matrix: Matrix4<f32>,

    // FIXME: do we need to store both original matrices and inverse matrices?
    inverse_view_matrix: Matrix4<f32>,
    inverse_proj_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        resolution_x: u32,
        resolution_y: u32,
        near_clip: f32,
        far_clip: f32,
        projection: Projection,
    ) -> Self {
        let mut camera = Self {
            position,
            target,
            up,
            resolution_x,
            resolution_y,
            near_clip,
            far_clip,
            projection,
            view_matrix: Matrix4::<_>::identity(),
            proj_matrix: Matrix4::<_>::identity(),
            inverse_view_matrix: Matrix4::<_>::identity(),
            inverse_proj_matrix: Matrix4::<_>::identity(),
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

    pub fn pan(&mut self, screen_delta: Vector2<f32>) {
        let delta_ndc = Vector2::new(
            (screen_delta.x / self.resolution_x() as f32) * 2.0,
            -(screen_delta.y / self.resolution_y() as f32) * 2.0,
        );
        let camera_to_target_distance = self.position().distance(self.target());
        let clip_space_delta = Vector4::new(delta_ndc.x, delta_ndc.y, 1.0, 0.0);

        let camera_space_delta = self.inverse_proj_matrix() * clip_space_delta;
        let camera_space_delta = camera_space_delta / camera_space_delta.w;

        let world_space_delta = self
            .inverse_view_matrix()
            .transform_vector(camera_space_delta.truncate());

        let pan_delta = world_space_delta * camera_to_target_distance;

        self.position += pan_delta;
        self.target += pan_delta;
        self.update_matrices();
    }

    pub fn target(&self) -> Point3<f32> {
        self.target
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
        self.update_matrices();
    }

    pub fn resolution_x(&self) -> u32 {
        self.resolution_x
    }

    pub fn set_resolution_x(&mut self, resolution_x: u32) {
        self.resolution_x = resolution_x;
        self.update_matrices();
    }

    pub fn resolution_y(&self) -> u32 {
        self.resolution_y
    }

    pub fn set_resolution_y(&mut self, resolution_x: u32) {
        self.resolution_y = resolution_x;
        self.update_matrices();
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.resolution_x as f32 / self.resolution_y as f32
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

    pub fn inverse_view_matrix(&self) -> Matrix4<f32> {
        self.inverse_view_matrix
    }

    pub fn inverse_proj_matrix(&self) -> Matrix4<f32> {
        self.inverse_proj_matrix
    }

    fn update_matrices(&mut self) {
        self.view_matrix = Matrix4::look_at_lh(self.position, self.target, self.up);

        let aspect_ratio = self.aspect_ratio();
        let near = self.near_clip;
        let far = self.far_clip;
        self.proj_matrix = match self.projection {
            Projection::Perspective { fov } => cgmath::perspective(fov, aspect_ratio, near, far),
            Projection::Orthographic { size } => cgmath::ortho(
                -size * aspect_ratio,
                size * aspect_ratio,
                -size,
                size,
                near,
                far,
            ),
        };

        // FIXME: what to do when the determinant is 0 (inverse matrix does not exist)
        self.inverse_view_matrix = self.view_matrix.invert().unwrap();
        self.inverse_proj_matrix = self.proj_matrix.invert().unwrap();
    }
}
