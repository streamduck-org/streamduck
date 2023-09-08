using System;

namespace Streamduck.Definitions.Inputs;

public interface IInputButton {
	event Action? ButtonPressed;
	event Action? ButtonReleased;
}