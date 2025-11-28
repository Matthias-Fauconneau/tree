use ui::{Result, vector::{vec3, mat4}, vulkan::{Arc, Subbuffer, Context, Commands, from_iter, BufferUsage, buffer, ImageView, PrimitiveTopology}, shader};
shader!{tree}

pub struct Tree {
	pass: tree::Pass,
	vertices: Subbuffer::<[tree::Vertex]>,
	indices: Subbuffer::<[u32]>,
}

impl Tree {
	pub fn new(context: &Context, quads: &[vec3]) -> Result<Self> {
		let vertices = from_iter(context, BufferUsage::VERTEX_BUFFER, quads.into_iter().map(|&p| tree::Vertex{position: p.into()}))?;
		let indices = buffer(context, BufferUsage::INDEX_BUFFER, quads.len()/4*6)?;
		{
			let mut indices = indices.write()?;
			for i in 0..quads.len()/4 {
				indices[i*6+0] = (i*4+0) as u32;
				indices[i*6+1] = (i*4+2) as u32;
				indices[i*6+2] = (i*4+1) as u32;
				indices[i*6+3] = (i*4+0) as u32;
				indices[i*6+4] = (i*4+3) as u32;
				indices[i*6+5] = (i*4+2) as u32;
			}
		}
		Ok(Self{
			pass: tree::Pass::new(context, true, PrimitiveTopology::TriangleList, false)?,
			vertices,
			indices,
		})
	}
	pub fn render(&mut self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) -> Result {
		let Self{pass, vertices, indices} = self;
		pass.begin_rendering(context, commands, color, Some(depth), true, &tree::Uniforms{
			view_projection: view_projection.map(|column| column.into()).into(),
		}, &[])?;
		commands.bind_index_buffer(indices.clone())?;
		commands.bind_vertex_buffers(0, vertices.clone())?;
		unsafe{commands.draw_indexed(indices.len() as _, 1, 0, 0, 0)}?;
		commands.end_rendering()?;
		Ok(())
	}
}
