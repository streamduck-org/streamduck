using System;
using System.Runtime.CompilerServices;
using System.Threading.Tasks;
using Streamduck.Definitions;
using Streamduck.Definitions.Inputs;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckButton : Input, IInputButton, IInputDisplay {
	private StreamDeckDevice _device;

	public StreamDeckButton(StreamDeckDevice device, int x, int y, UInt2 displayResolution)
		: base(x, y, 1, 1, InputIcon.Button) {
		_device = device;
		DisplayResolution = displayResolution;
	}

	public event Action? ButtonPressed;
	public event Action? ButtonReleased;

	public UInt2 DisplayResolution { get; }

	public Task ApplyImage(int key) => throw new NotImplementedException();

	public Task ClearImage() => throw new NotImplementedException();

	internal void CallPressed() {
		ButtonPressed?.Invoke();
	}

	internal void CallReleased() {
		ButtonReleased?.Invoke();
	}
}