using System;

namespace Streamduck.Definitions.Inputs;

public interface IInputToggle {
	bool ToggleState { get; }
	event Action<bool>? ToggleStateChanged;
}