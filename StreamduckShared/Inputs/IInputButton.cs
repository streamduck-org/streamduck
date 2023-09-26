using System;

namespace Streamduck.Inputs;

public interface IInputButton {
	event Action? ButtonPressed;
	event Action? ButtonReleased;
}