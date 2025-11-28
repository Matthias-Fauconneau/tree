#![feature(slice_from_ptr_range)] // shader
use ui::{Result, run, size, int2, Widget, vulkan, shader};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, image, WriteDescriptorSet, linear};
shader!{view}

pub fn default<T: Default>() -> T { Default::default() }
use core::f32;
use std::f32::consts::PI;
use {ui::{vector::{xy, xyz, vec2, vec3, cross, normalize, MinMax, minmax, rotate, sq}, image::{Image, rgb, sRGB8_EOTF, sRGB8_OETF12}}, rand::random};

struct App {
	lines: Vec<[vec3; 2]>,
	yaw: f32,
}

impl Widget for App {
fn paint(&mut self, context: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	let Self{lines, yaw} = self;
	let MinMax{min, max} = minmax(lines.iter().map(|&[a,b]| [a,b].into_iter()).flatten()).unwrap();
	let scale = 1. / ( xyz::from( max.x .max( max.y) .max( max.z ) ) - xyz::from( min.x .min( min.y ) .min( min.z ) ) );
	let pitch = -PI/3.;

	//let mut image = Image::zero(xy{x:3840,y:2400});
	let mut image = Image::zero(xy{x:1920,y:1200});
	let (eotf, oetf) = (&sRGB8_EOTF, &sRGB8_OETF12);
	for &[a,b] in lines.iter() {
		let [a,b] = [a,b].map(|p| {
			let xyz{x,y,z} = scale*(p-(min+max)/2.);
			let xy{x,y} = rotate(*yaw, xy{x,y});
			let xy{x: y, y: _z} = rotate(pitch, xy{x: y, y: z});
			vec2::from(image.size)/2. + xy{x, y: -y}*vec2::from(image.size.y as f32)
		});
		if a!=b { ui::line(eotf, oetf, &mut image.as_mut(), a, b, rgb{r: 1., g: 1., b: 1.}); }
	}

	let image = self::image(context, commands, image.as_ref())?;
	let mut pass = view::Pass::new(context, false, PrimitiveTopology::TriangleList, false)?;
		pass.begin_rendering(context, commands, target.clone(), None, true, &view::Uniforms::empty(), &[
		WriteDescriptorSet::image_view(0, ImageView::new_default(&image)?),
		WriteDescriptorSet::sampler(1, linear(context)),
	])?;
	unsafe{commands.draw(3, 1, 0, 0)}?;
	commands.end_rendering()?;
	*yaw += PI/60.;
	Ok(())
}
fn event(&mut self, _context: &Context, _commands: &mut Commands, _size: size, _event_context: &mut ui::EventContext, _event: &ui::Event) -> Result<bool> { Ok(true/*Keep repainting*/) }
}

fn main() -> Result {

	#[derive(Default)] struct Node {
		position: vec3,
		frame: [vec3; 3],
		radius: f32,
		nodes: Vec<Node>,
	}

	let mut tree = Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
		Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
			Node{position: xyz{x: 0., y: 0., z: 1.}, nodes: vec![
				Node{position: normalize(vec3{x: -1., y: 0., z: 1.}), ..default()},
				Node{position: normalize(vec3{x: 1., y: 0., z: 1.}), ..default()},
				Node{position: normalize(vec3{x: 0., y: -1., z: 1.}), ..default()},
				Node{position: normalize(vec3{x: 0., y: 1., z: 1.}), ..default()}
			], ..default()},
			Node{position: normalize(vec3{x: -1., y: 0., z: 1.}), ..default()},
			Node{position: normalize(vec3{x: 1., y: 0., z: 1.}), ..default()}
		], ..default()},
	], radius: 1./3., ..default()};

	fn depth_first(parent: &mut Node, f: &impl Fn(&mut Node)) {
		for child in &mut parent.nodes { depth_first(child, f); }
		f(parent);
	}
	for _ in 0..1 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: 0., z: 1.}, ..default()},
			Node{position: normalize(vec3{x: 0., y: 1., z: 1.}), ..default()}
		];})
	}
	for _ in 0..0 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: -1., z: 1.}, ..default()},
			Node{position: xyz{x: 0., y: 1., z: 1.}, ..default()}
		];})
	}

	fn walk_mut(parent: &mut Node, f: &mut impl FnMut(vec3, [vec3; 3], f32, &mut Node)) {
		for child in &mut parent.nodes { f(parent.position, parent.frame, parent.radius, child); walk_mut(child, f); }
	}
	let random_frame = |z| -> [vec3; 3] {
		let z = normalize(z);
		let x = normalize(cross(normalize(xyz{x: random::<f32>()*2.-1., y: random::<f32>()*2.-1., z: random::<f32>()*2.-1.}), z));
		let y = normalize(cross(z, x));
		[x,y,z]
	};
	tree.frame = random_frame(tree.position);
	assert_ne!(tree.radius, 0.);
	walk_mut(&mut tree, &mut |parent_position, [x,y,z]:[vec3; 3], radius, child| {
		let segment = child.position.x * x + child.position.y * y + child.position.z * z;
		child.position = parent_position + segment;
		child.frame = random_frame(segment);
		child.radius = radius / f32::cbrt(2.);
	});

	fn walk(parent: &Node, f: &mut impl FnMut(&Node, &Node)) {
		for child in &parent.nodes { f(parent, child); walk(child, f); }
	}

	//let mut lines = vec![([xyz{x: 0., y: 0., z: 0.}, tree.position], rgb{b: 1., g: 1., r: 1.}))];
	//walk(&tree, &mut |parent, child| lines.push(([parent.position, child.position], rgb{b: 1., g: 1., r: 1.})) );

	const N : usize = 7;
	impl Node {
		fn ring(&self) -> impl Iterator<Item=vec3> {
			let &Node{position, frame: [ex, ey, _], radius, ..} = self;
			(0..N).map(move |i| {
				let t = i as f32*2.*PI/N as f32;
				let (y,x) = t.sin_cos();
				let xy{x,y} = radius*xy{x,y};
				position+x*ex+y*ey
			})
		}
	}

	//let mut lines: Vec<_> = tree.ring().collect();
	//walk(&tree, &mut |_, branch| lines.extend(branch.ring()));

	let mut lines = vec![];
	walk(&tree, &mut |parent, branch| {
		let parent : Vec<_> = parent.ring().collect();
		let branch : Vec<_> = branch.ring().collect();
		let mut min = (0, 0, f32::MAX);
		for (a, parent) in parent.iter().enumerate() {
			for (b, branch) in branch.iter().enumerate() {
				let d = sq(parent-branch);
				if d < min.2 { min = (a, b, d); }
			}
		}
		for i in 0..N { lines.push([parent[(min.0+i)%N], branch[(min.1+i)%N]]); }
	});

	if lines.is_empty() { return Ok(()); }
	run("tree", Box::new(|_, _| Ok(Box::new(App{lines, yaw: 0.}))))
}
