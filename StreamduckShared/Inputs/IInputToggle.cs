using System;

namespace Streamduck.Inputs;

public interface IInputToggle {
	bool ToggleState { get; }
	event Action<bool>? ToggleStateChanged;
}