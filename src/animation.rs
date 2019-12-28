use ggez::{self, graphics::Image, timer};

use std::{collections::HashMap, fmt::Debug};

/// An animation that acts on a struct to provide a visual effect.
pub trait Animation<A>: Debug {
	fn update(&mut self,  _: &mut A, _: &mut ggez::Context) -> AnimationState;
	fn finish(&self, _: &mut A);
}

/// A struct that produces `Animation` trait objects.
///
/// The generic type A is the type of command it can be used for.
/// The `Parameter` type is the type of parameter the animation will accept.
pub trait AnimationProducer<A>: Debug {
	type Parameter;
	fn initialise(&self, _: A) -> Box<dyn Animation<Self::Parameter>>;
}

/// Stores all the [`AnimationProducer`]s, to be used at runtime.
#[derive(Debug)]
pub struct AnimationMap {
	/// Transitions that can be used for a `Change` Command.
	pub change: HashMap<String, Box<dyn AnimationProducer<ChangeAnimation, Parameter=InstanceParameter>>>,
	/// Transitions that can be used for a `Position` Command.
	pub position: HashMap<String, Box<dyn AnimationProducer<PositionAnimation, Parameter=InstanceParameter>>>,
	/// Transitions that can be used for a `Show` Command.
	pub show: HashMap<String, Box<dyn AnimationProducer<ShowAnimation, Parameter=InstanceParameter>>>,
	/// Transitions that can be used for a `Hide` Command.
	pub hide: HashMap<String, Box<dyn AnimationProducer<HideAnimation, Parameter=InstanceParameter>>>,
	/// Transitions that can be used for a `Spawn` Command.
	pub spawn: HashMap<String, Box<dyn AnimationProducer<SpawnAnimation, Parameter=InstanceParameter>>>,
	/// Transitions that can be used for a `Kill` Command.
	pub kill: HashMap<String, Box<dyn AnimationProducer<KillAnimation, Parameter=InstanceParameter>>>,
}
impl Default for AnimationMap {
	fn default() -> Self {
		let mut change = HashMap::with_capacity(1);
		let mut hide = HashMap::with_capacity(2);
		let mut kill = HashMap::with_capacity(2);
		let mut position = HashMap::with_capacity(1);
		let mut show = HashMap::with_capacity(2);
		let mut spawn = HashMap::with_capacity(2);
		change.insert("flip".into(), Box::new(Flip) as Box<_>);
		hide.insert("fade".into(), Box::new(Fade) as Box<_>);
		hide.insert("glide".into(), Box::new(Glide) as Box<_>);
		kill.insert("fade".into(), Box::new(Fade) as Box<_>);
		kill.insert("glide".into(), Box::new(Glide) as Box<_>);
		position.insert("glide".into(), Box::new(Glide) as Box<_>);
		show.insert("fade".into(), Box::new(Fade) as Box<_>);
		show.insert("glide".into(), Box::new(Glide) as Box<_>);
		spawn.insert("fade".into(), Box::new(Fade) as Box<_>);
		spawn.insert("glide".into(), Box::new(Glide) as Box<_>);
		Self { change, hide, kill, position, show, spawn }
	}
}

/// Declares what animation is to be used and the variable number of arguments it should be passed in.
#[derive(Debug)]
pub struct AnimationDeclaration {
	/// The name of the animation.
	pub name: String,
	/// A variable number of arguments that the animation will process.
	/// It is up to the animation writer to determine what the arguments are used for.
	pub arguments: Vec<Option<f32>>
}

/// The state of the animation.
pub enum AnimationState {
	/// The animation is ongoing.
	Continue,
	/// The animation has finished and should be removed.
	Finished
}

/// A parameter that represents the values of an [`Instance`] that will be given to an [`Animation`].
pub struct InstanceParameter {
	/// Position of the image centre in pixels.
	/// This determines the centre of rotation and scaling.
	pub centre_position: (f32, f32),
	/// Image that this instance draws to the screen.
	pub image: Image,
	/// Position on the screen in pixels.
	pub position: (f32, f32),
	/// Amount the image is scaled by.
	pub scale: (f32, f32),
	/// Whether the instance is visible.
	pub visible: bool,
	/// The colour of the instance.
	pub colour: [f32; 4],
}

/// An animation that is used on the `Position` Command will take in this struct.
///
/// When the animation finishes, the position of the [`Instance`]
/// **MUST** be the same value as the ``destination`` field.
pub struct PositionAnimation {
	/// The position where the [`Instance`] will eventually end up.
	pub destination: (f32, f32),
	/// Extra arguments provided to the Animation.
	pub arguments: Vec<Option<f32>>,
}

/// An animation that is used on the `Show` Command will take in this struct.
///
/// When the animation finishes, the visibility of the [`Instance`]
/// **MUST** be set to `true`
pub struct ShowAnimation {
	/// Extra arguments provided to the Animation.
	pub arguments: Vec<Option<f32>>,
}

/// An animation that is used on the `Hide` Command will take in this struct.
///
/// When the animation finishes, the visibility of the [`Instance`]
/// **MUST** be set to `false`
pub struct HideAnimation {
	/// Extra arguments provided to the Animation.
	pub arguments: Vec<Option<f32>>,
}

/// An animation that is used on the `Spawn` Command will take in this struct.
pub struct SpawnAnimation {
	/// Extra arguments provided to the Animation.
	pub arguments: Vec<Option<f32>>,
}

/// An animation that is used on the `Kill` Command will take in this struct.
pub struct KillAnimation {
	/// Extra arguments provided to the Animation.
	pub arguments: Vec<Option<f32>>,
}

/// An animation that is used on the `Change` Command will take in this struct.
///
/// When the ``finish`` method of the Animation is called,
/// the `centre_position` of the `Instance` must have the value of `new_centre_position`,
/// the `image` of the `Instance` must have the value of `new_image`, and
/// the `scale` of the `Instance` must have the value of `new_scale`.
pub struct ChangeAnimation {
	/// The new centre position that the instance is supposed to switch to by the end.
	pub new_centre_position: (f32, f32),
	/// The new image that the instance is supposed to switch to by the end.
	pub new_image: Image,
	/// The new scale that the instance is supposed to change to by the end.
	pub new_scale: (f32, f32),
	/// Extra arguments provided to the Animation.
	pub arguments: Vec<Option<f32>>,
}
impl ChangeAnimation {
	pub fn new(arguments: Vec<Option<f32>>, character: &super::CharacterName, script: &super::Script, state: &super::StateName) -> Self {
		let state = &script.characters[(character, state)];
		let new_image = script.images.get(&state.image).unwrap_or_else(||
			panic!("Image at path: {:?}, is not loaded", &state.image)).clone();
		let new_centre_position = state.centre_position.map(|(x, y)| (x as f32, y as f32))
			.unwrap_or_else(|| (new_image.width() as f32 / 2.0, new_image.height() as f32 / 2.0));
		let new_scale = state.scale;
		Self {
			new_centre_position,
			new_image,
			new_scale,
			arguments,
		}
	}
}

/// A Glide animation.
#[derive(Clone, Debug, Default)]
pub struct Glide;
impl AnimationProducer<PositionAnimation> for Glide {
	type Parameter = InstanceParameter;
	fn initialise(&self, animation: PositionAnimation) -> Box<dyn Animation<Self::Parameter>> {
        let time_period = animation.arguments.first().and_then(|period| *period).unwrap_or(10_000.0);
		Box::new(GlideMove{ destination: animation.destination, time_period })
	}
}
impl AnimationProducer<ShowAnimation> for Glide {
	type Parameter = InstanceParameter;
	fn initialise(&self, animation: ShowAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = animation.arguments.first().and_then(|period| *period).unwrap_or(10_000.0);
		// 0 is Left, 1 is Right, and it is 0 by default.
		let direction = match animation.arguments.get(1).and_then(|direction| *direction).unwrap_or(0.0) {
			d if d == 1.0 => GlideVisibilityDirection::Right,
			d if d == 0.0 => GlideVisibilityDirection::Left,
			_ => GlideVisibilityDirection::Left,
		};
		Box::new(GlideVisibility::Uninitialised(true, time_period, direction))
	}
}
impl AnimationProducer<HideAnimation> for Glide {
	type Parameter = InstanceParameter;
	fn initialise(&self, animation: HideAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = animation.arguments.first().and_then(|period| *period).unwrap_or(10_000.0);
		// 0 is Left, 1 is Right, and it is 0 by default.
		let direction = match animation.arguments.get(1).and_then(|direction| *direction).unwrap_or(0.0) {
			d if d == 1.0 => GlideVisibilityDirection::Right,
			d if d == 0.0 => GlideVisibilityDirection::Left,
			_ => GlideVisibilityDirection::Left,
		};
		Box::new(GlideVisibility::Uninitialised(false, time_period, direction))
	}
}
impl AnimationProducer<SpawnAnimation> for Glide {
	type Parameter = InstanceParameter;
	fn initialise(&self, animation: SpawnAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = animation.arguments.first().and_then(|period| *period).unwrap_or(10_000.0);
		// 0 is Left, 1 is Right, and it is 0 by default.
		let direction = match animation.arguments.get(1).and_then(|direction| *direction).unwrap_or(0.0) {
			d if d == 1.0 => GlideVisibilityDirection::Right,
			d if d == 0.0 => GlideVisibilityDirection::Left,
			_ => GlideVisibilityDirection::Left,
		};
		Box::new(GlideVisibility::Uninitialised(true, time_period, direction))
	}
}
impl AnimationProducer<KillAnimation> for Glide {
	type Parameter = InstanceParameter;
	fn initialise(&self, animation: KillAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = animation.arguments.first().and_then(|period| *period).unwrap_or(10_000.0);
		// 0 is Left, 1 is Right, and it is 0 by default.
		let direction = match animation.arguments.get(1).and_then(|direction| *direction).unwrap_or(0.0) {
			d if d == 1.0 => GlideVisibilityDirection::Right,
			d if d == 0.0 => GlideVisibilityDirection::Left,
			_ => GlideVisibilityDirection::Left,
		};
		Box::new(GlideVisibility::Uninitialised(true, time_period, direction))
	}
}

#[derive(Debug)]
struct GlideMove {
	destination: (f32, f32),
	time_period: f32,
}
impl Animation<InstanceParameter> for GlideMove {
	fn update(&mut self,  parameters: &mut InstanceParameter, ctx: &mut ggez::Context) -> AnimationState {
		let delta_time = (timer::duration_to_f64(timer::delta(ctx)) * 1_000.0) as f32;
		let time_left = self.time_period - delta_time;
		if self.time_period > 0. {
			let position_difference = (
				self.destination.0 - parameters.position.0,
				self.destination.1 - parameters.position.1
			);
			let position_delta = (
				position_difference.0 / time_left * 1_000.0,
				position_difference.1 / time_left * 1_000.0
			);
			parameters.position = (
				parameters.position.0 + position_delta.0,
				parameters.position.1 + position_delta.1
			);
			self.time_period = time_left;
			AnimationState::Continue
		} else {
			AnimationState::Finished
		}
	}
	fn finish(&self, parameters: &mut InstanceParameter) {
		parameters.position = self.destination;
	}
}

#[derive(Debug)]
enum GlideVisibility {
	Uninitialised(bool, f32, GlideVisibilityDirection),
	Initialised(bool, GlideMove, (f32, f32)),
}

#[derive(Debug)]
enum GlideVisibilityDirection {
	Left,
	Right,
}

impl GlideVisibility {
	fn initialise(&mut self, parameter: &mut InstanceParameter) {
		if let GlideVisibility::Uninitialised(visible, time_period, direction) = self {
			let width = parameter.image.width() as f32;
			let destination_x;
			let original_x = parameter.position.0;
			if *visible {
				destination_x = original_x;
				parameter.position.0 = match direction {
					GlideVisibilityDirection::Left => -(width-parameter.centre_position.0),
					GlideVisibilityDirection::Right => {
						let screen_width = 640.0; // To Do: This value is currently hard coded to be the same as Settings::default.
						screen_width + width - parameter.centre_position.0
					}
				};
			} else {
				destination_x = match direction {
					GlideVisibilityDirection::Left => -(width-parameter.centre_position.0),
					GlideVisibilityDirection::Right => {
						let screen_width = 640.0; // To Do: This value is currently hard coded to be the same as Settings::default.
						screen_width + width - parameter.centre_position.0
					}
				};
			}
			*self = GlideVisibility::Initialised(*visible, GlideMove {
				destination: (destination_x, parameter.position.1),
				time_period: *time_period,
			}, (original_x, parameter.position.1));
		}
	}
}
impl Animation<InstanceParameter> for GlideVisibility {
	fn update(&mut self, parameter: &mut InstanceParameter, ctx: &mut ggez::Context) -> AnimationState {
		parameter.visible = true;
		match self {
			GlideVisibility::Uninitialised(_, _, _) => {
				self.initialise(parameter);
				self.update(parameter, ctx)
			}
			GlideVisibility::Initialised(_, glide_move, _) => glide_move.update(parameter, ctx),
		}
	}
	fn finish(&self, parameter: &mut InstanceParameter) {
		parameter.visible = match self {
			GlideVisibility::Initialised(visible, _, _) | GlideVisibility::Uninitialised(visible, _, _) => *visible,
		};
		if let GlideVisibility::Initialised(_, _, position) = self {
			parameter.position = *position;
		}
	}
}

#[derive(Debug)]
pub struct Fade;
impl AnimationProducer<ShowAnimation> for Fade {
	type Parameter = InstanceParameter;
	fn initialise(&self, parameters: ShowAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = parameters.arguments.first().and_then(|period| *period).unwrap_or(250.0);
		let rate = time_period.recip();
		Box::new(FadeVisibility { alpha: 0.0, time_period, rate, visibility: true }) as Box<_>
	}
}
impl AnimationProducer<HideAnimation> for Fade {
	type Parameter = InstanceParameter;
	fn initialise(&self, parameters: HideAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = parameters.arguments.first().and_then(|period| *period).unwrap_or(250.0);
		let rate = -time_period.recip();
		Box::new(FadeVisibility { alpha: 1.0, time_period, rate, visibility: false }) as Box<_>
	}
}
impl AnimationProducer<SpawnAnimation> for Fade {
	type Parameter = InstanceParameter;
	fn initialise(&self, parameters: SpawnAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = parameters.arguments.first().and_then(|period| *period).unwrap_or(250.0);
		let rate = time_period.recip();
		Box::new(FadeVisibility { alpha: 0.0, time_period, rate, visibility: true }) as Box<_>
	}
}
impl AnimationProducer<KillAnimation> for Fade {
	type Parameter = InstanceParameter;
	fn initialise(&self, parameters: KillAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let time_period = parameters.arguments.first().and_then(|period| *period).unwrap_or(250.0);
		let rate = -time_period.recip();
		Box::new(FadeVisibility { alpha: 1.0, time_period, rate, visibility: false }) as Box<_>
	}
}

/// An animation that works for both the Show and Hide command.
#[derive(Debug)]
struct FadeVisibility {
	/// How long this animation will last in ms.
	time_period: f32,
	/// Rate of opacity change per ms.
	rate: f32,
	/// The *intended* visibility at the end of the transition.
	visibility: bool,
	/// The current alpha value of the instance.
	alpha: f32,
}
impl Animation<InstanceParameter> for FadeVisibility {
	fn update(&mut self, parameter: &mut InstanceParameter, ctx: &mut ggez::Context) -> AnimationState {
		let delta_time = (timer::duration_to_f64(timer::delta(ctx)) * 1_000.0) as f32;
		self.time_period -= delta_time;
		if self.time_period > 0.0 {
			self.alpha += self.rate * delta_time;
			parameter.colour[3] = self.alpha;
			parameter.visible = true;
			AnimationState::Continue
		} else {
			AnimationState::Finished
		}
	}
	fn finish(&self, parameter: &mut InstanceParameter) {
		parameter.colour[3] = 1.0;
		parameter.visible = self.visibility;
	}
}

#[derive(Debug)]
pub struct Flip;
impl AnimationProducer<ChangeAnimation> for Flip {
	type Parameter = InstanceParameter;
	fn initialise(&self, parameter: ChangeAnimation) -> Box<dyn Animation<Self::Parameter>> {
		let ChangeAnimation{ new_centre_position, new_image, new_scale, arguments } = parameter;
		let time_period = arguments.first().and_then(|o| *o).unwrap_or(100.0);
		let time_left = time_period;
		Box::new(FlipChange{ time_period, time_left, new_centre_position, new_image, new_scale, original_scale: None })
	}
}

#[derive(Debug)]
struct FlipChange {
	time_period: f32,
	time_left: f32,
	new_centre_position: (f32, f32),
	new_image: Image,
	new_scale: (f32, f32),
	original_scale: Option<(f32, f32)>,
}
impl Animation<InstanceParameter> for FlipChange {
	fn update(&mut self, parameter: &mut InstanceParameter, ctx: &mut ggez::Context) -> AnimationState {
		if self.original_scale.is_none() {
			self.original_scale = Some(parameter.scale);
		}
		let delta_time = (timer::duration_to_f64(timer::delta(ctx)) * 1_000.0) as f32;
		if self.time_left > 0.0 {
			self.time_left -= delta_time;
			if self.time_left<= 0.0 {
				parameter.image = self.new_image.clone();
				parameter.centre_position = self.new_centre_position;
				self.original_scale = Some(self.new_scale);
			}
		} else if self.time_left <= -self.time_period {
			return AnimationState::Finished
		} else {
			self.time_left -= delta_time;
		}
		parameter.scale.0 = self.original_scale.unwrap().0 * self.time_left.abs() / self.time_period;
		AnimationState::Continue
	}
	fn finish(&self, parameter: &mut InstanceParameter) {
		parameter.image = self.new_image.clone();
		parameter.centre_position = self.new_centre_position;
		parameter.scale = self.new_scale;
	}
}
