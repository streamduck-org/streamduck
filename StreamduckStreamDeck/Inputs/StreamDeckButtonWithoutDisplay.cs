using System;
using Streamduck.Inputs;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckButtonWithoutDisplay(int x, int y) : Input(x, y, 1, 1, InputIcon.Button), IInputButton {
	public event Action? ButtonPressed;
	public event Action? ButtonReleased;

	internal void CallPressed() {
		ButtonPressed?.Invoke();
	}

	internal void CallReleased() {
		ButtonReleased?.Invoke();
	}
}