using System;
using Streamduck.Inputs;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckEncoder : Input, IInputButton, IInputEncoder {
	public StreamDeckEncoder(int x, int y) : base(x, y, 1, 1, InputIcon.Encoder) { }

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