use cgmath::{Point3, Rad};
use super::camera::Camera;

const VERTICAL_OFFSET: f32 = 2.0;

pub struct OrbitController {
    target: Point3<f32>,
    distance: f32,
    yaw: f32,
    pitch: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
}

impl OrbitController {

    pub fn new() -> Self {
        Self {
            target: Point3::new(0.0, 0.0, 0.0),
            distance: 10.0,
            yaw: 0.0,
            pitch: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
        }
    }

    pub fn process_mouse(&mut self, delta_x: f64, delta_y: f64) {
        self.rotate_horizontal = delta_x as f32;
        self.rotate_vertical = delta_y as f32;
    }

    pub fn update_target(&mut self, mut target: [f32; 3]) {
        target[1] += VERTICAL_OFFSET;
        self.target = Point3::from(target);
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        // Mise à jour des angles yaw et pitch avec les rotations enregistrées
        self.yaw += self.rotate_horizontal * 0.01; // Sensibilité de la rotation
        self.pitch += self.rotate_vertical * 0.01; // Sensibilité de la rotation

        // Limiter l'angle de pitch pour éviter de passer sous ou au-dessus du personnage
        const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.1; // Limiter à presque 90°
        self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

        // Calculer la position de la caméra en coordonnées sphériques
        // let x = self.distance * self.yaw.cos() * self.pitch.cos();
        // let y = self.distance * self.pitch.sin();
        // let z = self.distance * self.yaw.sin() * self.pitch.cos();

        let horizontal_distance = self.distance * self.pitch.cos(); // Distance projetée sur le plan XZ
        let x = horizontal_distance * self.yaw.cos(); // Composante X du déplacement
        let z = horizontal_distance * self.yaw.sin(); // Composante Z du déplacement
        let y = self.distance * self.pitch.sin(); // Composante verticale du déplacement (rotation verticale)


        // Mettre à jour la position de la caméra en fonction de la cible (personnage)
        camera.position = Point3::new(
            self.target.x + x,
            self.target.y + y,
            self.target.z + z,
        );

        // Mettre à jour l'orientation de la caméra (yaw et pitch)
        camera.yaw = Rad(self.yaw + std::f32::consts::PI);
        camera.pitch = Rad(-self.pitch);

        // Réinitialiser les rotations après l'update
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }

}