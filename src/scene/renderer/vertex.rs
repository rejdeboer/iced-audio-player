#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn empty() -> Self {
        Vertex {
            position: [0f32, 0f32],
        }
    }
}

pub fn generate_spectrum_vertices(data: &[f32]) -> Vec<Vertex> {
    let mut vertices: Vec<Vertex> = vec![];

    for (i, vertex) in data.iter().enumerate() {
        let frac = i as f32 / data.len() as f32;
        let x = (2.0 * frac) - 1f32;
        let y = (2.0 * vertex) - 1f32;

        vertices.push(Vertex { position: [x, -1.] });
        vertices.push(Vertex { position: [x, y] });
    }

    vertices
}
