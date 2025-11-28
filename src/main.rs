#![feature(slice_from_ptr_range)] // shader
use ui::{Result, run, xy, size, int2, image::{self, Image}, Widget, vulkan, shader};
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
	use {ui::vector::{self, xyz, vec2, vec3, cross, normalize}, image::rgb};

	struct Node {
		position: vec3,
		nodes: Vec<Node>,
	}

	let mut tree = Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
		Node{position: xyz{x: 1., y: 0., z: 0.}, nodes: vec![]},
		Node{position: xyz{x: 0., y: 1., z: 0.}, nodes: vec![]}
	]};

	/*let mut tree = Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
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
	for _ in 0..1 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 0., y: 1., z: 1.}, nodes: vec![]}
		];})
	}
	for _ in 0..0 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: -1., z: 1.}, nodes: vec![]},
			Node{position: xyz{x: 0., y: 1., z: 1.}, nodes: vec![]}
		];})
	}*/

	fn walk_mut<T:Copy>(parent: &mut Node, f: &mut impl FnMut(vec3, &mut Node, T)->T, r: T) {
		for child in &mut parent.nodes { let r = f(parent.position, child, r); walk_mut(child, f, r); }
	}
	let root_direction = tree.position;

	let mut lines = vec![];
	lines.push(([xyz{x: 0., y: 0., z: 0.}.xy(), tree.position.xy()], rgb{b: 1., g: 1., r: 1.}));

	walk_mut(&mut tree, &mut |parent_position, child, parent_direction_z| {
		let mut parent_direction_x = cross(xyz{x: 0., y: 1., z: 0.}, parent_direction_z);
		if vector::sq(parent_direction_x) <= f32::EPSILON { parent_direction_x = cross(xyz{x: 1., y: 0., z: 0.}, parent_direction_z); }
		let parent_direction_y = cross(parent_direction_z, parent_direction_x);
		println!("{parent_position:?} {parent_direction_x:?} {parent_direction_y:?} {parent_direction_z:?}");
		lines.push(([parent_position.xy(), (parent_position+parent_direction_x).xy()], rgb{r: 1., g: 0., b: 0.}));
		lines.push(([parent_position.xy(), (parent_position+parent_direction_y).xy()], rgb{r: 0., g: 1., b: 0.}));
		lines.push(([parent_position.xy(), (parent_position+parent_direction_z).xy()], rgb{r: 0., g: 0., b: 1.}));
		let segment = child.position.x * parent_direction_x + child.position.y * parent_direction_y + child.position.z * parent_direction_z;
		child.position = parent_position + segment;
		normalize(segment)
	}, /*xyz{x: 0., y: 0., z: 1.}*/root_direction);

	fn walk(parent: &Node, f: &mut impl FnMut(&Node, &Node)) {
		for child in &parent.nodes { f(parent, child); walk(child, f); }
	}
	/*let mut lines = vec![];
	lines.push(([xyz{x: 0., y: 0., z: 0.}.xy(), tree.position.xy()], rgb{b: 1., g: 1., r: 1.}));*/
	walk(&tree, &mut |parent, child| lines.push(([parent.position.xy(), child.position.xy()], rgb{b: 1., g: 1., r: 1.})) );
	if true {
		//println!("{lines:?}");
		Ok(())
	} else {
		let vector::MinMax{min, max} = vector::minmax(lines.iter().map(|&([a,b],_)| [a,b].into_iter()).flatten()).unwrap();
		let mut image = Image::zero(xy{x:3840,y:2400});
		let (eotf, oetf) = (&image::sRGB8_EOTF, &image::sRGB8_OETF12);
		for ([a,b], c) in lines {
			let [a,b] = [a,b].map(|p| (p-min)/(max-min)*vec2::from(image.size));
			if a!=b { ui::line(eotf, oetf, &mut image.as_mut(), a, b, c); }
		}
		run("tree", Box::new(|context, commands| Ok(Box::new(App(self::image(context, commands, image.as_ref())?)))))
	}
}
