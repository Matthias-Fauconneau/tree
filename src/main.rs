#![feature(slice_from_ptr_range)] // shader
use ui::{Result, run, size, int2, image::{self, Image, rgba8}, Widget, vulkan, shader};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, image, WriteDescriptorSet, linear};
shader!{view}

struct App;

impl Widget for App {
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, size: size, _: int2) -> Result {
	let mut pass = view::Pass::new(context, false, PrimitiveTopology::TriangleList, false)?;

	use {ui::vector::{self, xyz, vec2, vec3}, image::rgb};

	//#[derive(Debug)]
	struct Node {
		position: vec3,
		//name: &'static str,
		nodes: Vec<Node>,
	}
	let mut tree = Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
		Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
			Node{position: xyz{x: -1., y: 0., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 1., y: 0., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 0., y: -1., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 0., y: 1., z: 1.}, nodes: vec![]}
		]},
		Node{position: xyz{x: -1., y: 0., z: 1.}, nodes: vec![]},
		Node{position: xyz{x: 1., y: 0., z: 1.}, nodes: vec![]}
	]};

	fn depth_first(parent: &mut Node, f: &impl Fn(&mut Node)) {
		for child in &mut parent.nodes { depth_first(child, f); }
		f(parent);
	}
	for _ in 0..4 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 0., y: 1., z: 1.}, nodes: vec![]}
		];})
	}
	for _ in 0..4 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: -1., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 0., y: 1., z: 1.}, nodes: vec![]}
		];})
	}

	fn walk_mut(parent: &mut Node, f: &impl Fn(vec3, &mut Node)) {
		for child in &mut parent.nodes { f(parent.position, child); walk_mut(child, f); }
	}
	walk_mut(&mut tree, &|parent, child| { child.position += parent; });

	fn walk(parent: &Node, f: &mut impl FnMut(&Node, &Node)) {
		for child in &parent.nodes { f(parent, child); walk(child, f); }
	}
	let mut lines = vec![];
	lines.push([xyz{x: 0., y: 0., z: 0.}.xz(), tree.position.xz()]);
	walk(&tree, &mut |parent, child| lines.push([parent.position.xz(), child.position.xz()]) );
	let vector::MinMax{min, max} = vector::minmax(lines.iter().map(|&[a,b]| [a,b].into_iter()).flatten()).unwrap();

	let mut image = Image::zero(size);

	let (eotf, oetf) = (&image::sRGB8_EOTF, &image::sRGB8_OETF12);
	for [a,b] in lines {
		let [a,b] = [a,b].map(|p| (p-min)/(max-min)*vec2::from(image.size));
		ui::line(eotf, oetf, &mut image.as_mut(), a, b, rgb{b: 1., g: 1., r: 1.});
	}

	let image = self::image(context, commands, image.as_ref())?;
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
