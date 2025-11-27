#![feature(slice_from_ptr_range)] // shader
use ui::{Result, run, size, int2, image::{Image, xy, rgba8}, Widget, vulkan, shader};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, image, WriteDescriptorSet, linear};
shader!{view}

struct App;

impl Widget for App { 
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	let mut pass = view::Pass::new(context, false, PrimitiveTopology::TriangleList, false)?;

	use ui::vector::{xyz, vec3};

	#[derive(Debug)] struct Node {
		position: vec3,
		name: &'static str,
		nodes: Vec<Node>,
	}
	let mut tree = Node{position: xyz{x: 0., y: 0., z: 1.}, name: "root", nodes: vec![
		Node{position: xyz{x: 0., y: 0., z: 1.}, name: "trunk", nodes: vec![
			Node{position: xyz{x: -1., y: 0., z: 1.}, name: "-X", nodes: vec![]},
			Node{position: xyz{x: 1., y: 0., z: 1.}, name: "+X", nodes: vec![]},
			Node{position: xyz{x: 0., y: -1., z: 1.}, name: "-Y", nodes: vec![]},
			Node{position: xyz{x: 0., y: 1., z: 1.}, name: "+Y", nodes: vec![]}
		]},
		Node{position: xyz{x: -1., y: 0., z: 1.}, name: "-X", nodes: vec![]},
		Node{position: xyz{x: 1., y: 0., z: 1.}, name: "+X", nodes: vec![]}
	]};	
	fn depth_first(parent: &mut Node, f: &impl Fn(&mut Node)) {
		for child in &mut parent.nodes { depth_first(child, f); }
		f(parent);
	}
	for _ in 0..4 {
		depth_first(&mut tree, &|Node{nodes: branches,..}:&mut Node| { if branches.is_empty() { branches = vec![Node{name: "main", nodes: vec![]}, Node{name: "side", nodes: vec![]}]; }})
	}
	for _ in 0..4 {
		depth_first(&mut tree, &|Node{nodes: branches,..}:&mut Node| { if branches.is_empty() { branches = vec![Node{name: "0", nodes: vec![]}, Node{name: "1", nodes: vec![]}]; }})
	}
	println!("{tree:?}");

	Image::from_xy(xy{x: 16, y: 16}, |xy{x,y}| rgba8{r: if x%2==0 { 0 } else { 0xFF }, g: if y%2==0 { 0 } else { 0xFF }, b: 0xFF, a: 0xFF})

	let image = image(context, commands, image.as_ref())?;
	pass.begin_rendering(context, commands, target.clone(), None, true, &view::Uniforms::empty(), &[
		WriteDescriptorSet::image_view(0, ImageView::new_default(&image)?),
		WriteDescriptorSet::sampler(1, linear(context)),
	])?;
	unsafe{commands.draw(3, 1, 0, 0)}?;
	commands.end_rendering()?;
	Ok(())
}
}

fn main() -> Result { run("tree", Box::new(|_,_| Ok(Box::new(App)))) }
