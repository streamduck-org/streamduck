using System;
using Streamduck.Inputs;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckEncoder(int x, int y) : Input(x, y, 1, 1, InputIcon.Encoder), IInputButton, IInputEncoder {
	public event Action? ButtonPressed;
	public event Action? ButtonReleased;
	public event Action<int>? EncoderTwisted;

	internal void CallPressed() {
		ButtonPressed?.Invoke();
	}

	internal void CallReleased() {
		ButtonReleased?.Invoke();
	}

	internal void CallTwist(int value) {
		EncoderTwisted?.Invoke(value);
	}
}