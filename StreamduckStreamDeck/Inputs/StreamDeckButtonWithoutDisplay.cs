using System;
using Streamduck.Definitions.Inputs;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckButtonWithoutDisplay : Input, IInputButton {
	public StreamDeckButtonWithoutDisplay(int x, int y) : base(x, y, 1, 1, InputIcon.Button) { }

	public event Action? ButtonPressed;
	public event Action? ButtonReleased;

	internal void CallPressed() {
		ButtonPressed?.Invoke();
	}

	internal void CallReleased() {
		ButtonReleased?.Invoke();
	}
}