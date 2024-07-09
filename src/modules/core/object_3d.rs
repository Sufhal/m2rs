pub struct Object3D<'a> {
    matrix: cgmath::Matrix4<f32>,
    childrens: Vec<&'a Object3D<'a>>,
    parent: Option<&'a Object3D<'a>>
}

impl Object3D<'_> {
    
}