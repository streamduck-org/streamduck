namespace Streamduck.Definitions.Inputs;

public enum InputBehavior {
	/**
	 * Input can be pressed down
	 */
	Button,

	/**
	 * Input can tell press pressure
	 */
	Pressure,

	/**
	 * Input can be toggled on or off
	 */
	Toggle,

	/**
	 * Input can be moved in one dimension
	 */
	Slider,

	/**
	 * Input can be turned in each direction to a certain limit
	 */
	Knob,

	/**
	 * Input provides relative values when turned
	 */
	Encoder,

	/**
	 * Input can be pressed down with 2D positional data
	 */
	TouchScreen,

	/**
	 * Input provides relative 2D values when used (encoder but 2D)
	 */
	Trackpad,

	/**
	 * Input provides precise 2D values when used
	 */
	Joystick
}