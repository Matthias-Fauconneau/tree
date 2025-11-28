#![feature(slice_from_ptr_range)] // shader
#![feature(iter_array_chunks)] // tree
mod tree; use tree::Tree;
use ui::{Result, run, size, int2, Widget, vulkan};
use vulkan::{Context, Commands, Arc, ImageView, Image as GPUImage, ImageCreateInfo, Format, ImageUsage};

pub fn default<T: Default>() -> T { Default::default() }
use core::f32;
use std::f32::consts::PI;
use {ui::{vector::{xy, xyz, xyzw, vec3, vec4, mat4, cross, normalize, MinMax, minmax, rotate, sq}}, rand::random};

struct App {
	tree: Tree,
	scale: f32,
	yaw: f32,
}

impl Widget for App {
fn paint(&mut self, context@Context{memory_allocator, ..}: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	let Self{tree, scale, yaw} = self;
	let image_size = {let [x,y,_] = target.image().extent(); xy{x,y}};
	let aspect_ratio = image_size.x as f32/image_size.y as f32;

	let pitch = -PI/3.;

	let view_projection = |p:vec3| {
		let xyz{x,y,z} = *scale*(p-xyz{x:0.,y:0.,z:1.});
		let xy{x,y} = rotate(*yaw, xy{x,y});
		let xy{x: y, y: _z} = rotate(pitch, xy{x: y, y: z});
		let z = (z-1.)/2.;
		xyzw{x, y: aspect_ratio*y, z: -z, w: 1.}
	};
	fn from_linear(linear: impl Fn(vec3)->vec4) -> mat4 {
		let w = linear(xyz{x:0.,y:0.,z:0.});
		let xyz{x,y,z} = xyz{x: xyz{x:1.,y:0.,z:0.}, y:xyz{x:0.,y:1.,z:0.}, z: xyz{x:0.,y:0.,z:1.}}.map(|e| linear(e)-w);
		xyzw{x,y,z,w}
	}
	let view_projection = from_linear(view_projection);

	let depth = ImageView::new_default(&GPUImage::new(&memory_allocator, &ImageCreateInfo{
		format: Format::D16_UNORM,
		extent: target.image().extent(),
		usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
		..default()
	}, &default())?)?;

	tree.render(context, commands, target.clone(), depth.clone(), view_projection)?;

	*yaw += PI/60./6.;
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

	let mut tree = Node{position: xyz{x: 0., y: 0., z: 0.1}, nodes: vec![
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
	for _ in 0..2 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: xyz{x: 0., y: 0., z: 1.}, ..default()},
			Node{position: normalize(vec3{x: 0., y: 1., z: 1.}), ..default()}
		];})
	}
	for _ in 0..1 {
		depth_first(&mut tree, &|Node{nodes,..}:&mut Node| if nodes.is_empty() { *nodes = vec![
			Node{position: normalize(vec3{x: 0., y: -1., z: 1.}), ..default()},
			Node{position: normalize(vec3{x: 0., y: 1., z: 1.}), ..default()}
		];})
	}

	fn leaves_mut(node: &mut Node, f: &impl Fn(&mut Node)) {
		if node.nodes.is_empty() { f(node); }
		else { for child in &mut node.nodes { leaves_mut(child, f); } }
	}
	leaves_mut(&mut tree, &|leaf| leaf.nodes = vec![Node{position: xyz{x: 0., y: 0., z: 0.1}, ..default()}]);

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

	leaves_mut(&mut tree, &|leaf| leaf.radius = 0.);

	fn walk(parent: &Node, f: &mut impl FnMut(&Node, &Node)) {
		for child in &parent.nodes { f(parent, child); walk(child, f); }
	}

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

	let mut quads = vec![];
	walk(&tree, &mut |parent, branch| { // FIXME: deduplicate vertices
		let parent : Vec<_> = parent.ring().collect();
		let branch : Vec<_> = branch.ring().collect();
		let mut min = (0, 0, f32::MAX);
		for (a, parent) in parent.iter().enumerate() {
			for (b, branch) in branch.iter().enumerate() {
				let d = sq(parent-branch);
				if d < min.2 { min = (a, b, d); }
			}
		}
		for i in 0..N { quads.extend_from_slice(&[parent[(min.0+i)%N], branch[(min.1+i)%N], branch[(min.1+i+1)%N], parent[(min.0+i+1)%N]]); }
	});

	let MinMax{min, max} = minmax(quads.iter().copied()).unwrap();
	let scale = 1. / ( ( max.x .max( max.y) .max( max.z ) ) - ( min.x .min( min.y ) .min( min.z ) ) );

	run("tree", Box::new(|context, _| Ok(Box::new(App{tree: Tree::new(context, &quads)?, scale, yaw: 0.}))))
}