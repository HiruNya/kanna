use ggez::{self, timer};

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
	/// Transitions that can be used for a `Position` Command.
	pub position: HashMap<String, Box<dyn AnimationProducer<PositionAnimation, Parameter=InstanceParameter>>>
}
impl Default for AnimationMap {
	fn default() -> Self {
		let mut position = HashMap::with_capacity(1);
		position.insert("glide".into(), Box::new(Glide) as Box<_>);
		Self { position }
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
	/// Position on the screen in pixels.
	pub position: (f32, f32),
	/// Amount the image is scaled by.
	pub scale: (f32, f32),
	/// Whether the instance is visible.
	pub visible: bool,
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

/// A Glide animation.
#[derive(Clone, Debug, Default)]
pub struct Glide;
impl AnimationProducer<PositionAnimation> for Glide {
	type Parameter = InstanceParameter;
	fn initialise(&self, animation: PositionAnimation) -> Box<dyn Animation<Self::Parameter>> {
        let time_period = animation.arguments.first().and_then(|period| *period).unwrap_or(1.0);
		Box::new(GlideAnimation::new(animation.destination, time_period))
	}
}

#[derive(Debug)]
struct GlideAnimation {
	destination: (f32, f32),
	time_period: f32,
}
impl GlideAnimation {
	/// Create a new glide animation.
	/// `time_period` is in milliseconds.
	fn new(destination: (f32, f32), time_period: f32) -> Self {
		Self {
			destination,
			time_period,
		}
	}
}
impl Animation<InstanceParameter> for GlideAnimation {
	fn update(&mut self,  parameters: &mut InstanceParameter, ctx: &mut ggez::Context) -> AnimationState {
		let delta_time = (timer::duration_to_f64(timer::delta(ctx)) / 1_000.) as f32;
		let time_left = self.time_period - delta_time;
		if self.time_period > 0. {
			let position_difference = (
				self.destination.0 - parameters.position.0,
				self.destination.1 - parameters.position.1
			);
			let position_delta = (
				position_difference.0 / time_left,
				position_difference.1 / time_left
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
