using System;

namespace Streamduck.Definitions.Inputs;

public interface IInputJoystick {
	Double2 JoystickValue { get; }
	Double2 MinJoystickValue { get; }
	Double2 MaxJoystickValue { get; }
	event Action<Double2>? JoystickValueChanged;
}