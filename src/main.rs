#![feature(slice_from_ptr_range)] // shader
use ui::{Result, run, size, int2, Widget, vulkan, shader};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, image, Image as GPUImage,WriteDescriptorSet, linear};
shader!{view}

struct App(Arc<GPUImage>);

impl Widget for App {
fn paint(&mut self, context: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	let mut pass = view::Pass::new(context, false, PrimitiveTopology::TriangleList, false)?;
		pass.begin_rendering(context, commands, target.clone(), None, true, &view::Uniforms::empty(), &[
		WriteDescriptorSet::image_view(0, ImageView::new_default(&self.0)?),
		WriteDescriptorSet::sampler(1, linear(context)),
	])?;
	unsafe{commands.draw(3, 1, 0, 0)}?;
	commands.end_rendering()?;
	Ok(())
}
}

fn main() -> Result {
	use {ui::{vector::{num::sqrt, xy, xyz, vec2, vec3, cross, normalize, MinMax, minmax}, image::{Image, rgb, sRGB8_EOTF, sRGB8_OETF12}}, rand::random};

	struct Node {
		position: vec3,
		nodes: Vec<Node>,
	}

	/*let mut tree = Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
		Node{position: xyz{x: 1., y: 0., z: 0.}, nodes: vec![Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![]}]},
		Node{position: xyz{x: 0., y: 1., z: 0.}, nodes: vec![Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![]}]}
	]};*/

	let mut tree = Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
		Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
			Node{position: normalize(vec3{x: -1., y: 0., z: 1.}), nodes: vec![]},
			Node{position: normalize(vec3{x: 1., y: 0., z: 1.}), nodes: vec![]},
			Node{position: normalize(vec3{x: 0., y: -1., z: 1.}), nodes: vec![]},
			Node{position: normalize(vec3{x: 0., y: 1., z: 1.}), nodes: vec![]}
		]},
		Node{position: normalize(vec3{x: -1., y: 0., z: 1.}), nodes: vec![]},
		Node{position: normalize(vec3{x: 1., y: 0., z: 1.}), nodes: vec![]}
	]};

	fn depth_first(parent: &mut Node, f: &impl Fn(&mut Node)) {
		for child in &mut parent.nodes { depth_first(child, f); }
		f(parent);
	}
	for _ in 0..1 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![]},
			Node{position: normalize(vec3{x: 0., y: 1., z: 1.}), nodes: vec![]}
		];})
	}
	for _ in 0..0 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: -1., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 0., y: 1., z: 1.}, nodes: vec![]}
		];})
	}

	fn walk_mut<T:Copy>(parent: &mut Node, f: &mut impl FnMut(vec3, &mut Node, T)->T, r: T) {
		for child in &mut parent.nodes { let r = f(parent.position, child, r); walk_mut(child, f, r); }
	}
	let random_frame = |z| -> [vec3; 3] {
		let z = normalize(z);
		let x = normalize(cross(normalize(xyz{x: random::<f32>()*2.-1., y: random::<f32>()*2.-1., z: random::<f32>()*2.-1.}), z));
		let y = normalize(cross(z, x));
		[x,y,z]
	};
	let root_direction = tree.position;
	walk_mut(&mut tree, &mut |parent_position, child, [x,y,z]:[vec3; 3]| {
		let segment = child.position.x * x + child.position.y * y + child.position.z * z;
		child.position = parent_position + segment;
		random_frame(segment)
	}, random_frame(root_direction));

	fn walk(parent: &Node, f: &mut impl FnMut(&Node, &Node)) {
		for child in &parent.nodes { f(parent, child); walk(child, f); }
	}
	let mut lines = vec![];
	lines.push(([xyz{x: 0., y: 0., z: 0.}.xy(), tree.position.xy()], rgb{b: 1., g: 1., r: 1.}));
	walk(&tree, &mut |parent, child| lines.push(([parent.position.xy(), child.position.xy()], rgb{b: 1., g: 1., r: 1.})) );
	let MinMax{min, max} = minmax(lines.iter().map(|&([a,b],_)| [a,b].into_iter()).flatten()).unwrap();
	let min = xy::from( min.x .min( min.y ) );
	let max = xy::from( max.x .max( max.y) );
	let mut image = Image::zero(xy{x:3840,y:2400});
	let (eotf, oetf) = (&sRGB8_EOTF, &sRGB8_OETF12);
	for ([a,b], c) in lines {
		let [a,b] = [a,b].map(|p| xy{x: (image.size.x-image.size.y) as f32/2., y: 0.} + (p-min)/(max-min)*vec2::from(image.size.y as f32));
		if a!=b { ui::line(eotf, oetf, &mut image.as_mut(), a, b, c); }
	}
	run("tree", Box::new(|context, commands| Ok(Box::new(App(self::image(context, commands, image.as_ref())?)))))
}
