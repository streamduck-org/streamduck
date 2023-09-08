using System;
using System.Threading.Tasks;
using Streamduck.Definitions;
using Streamduck.Definitions.Inputs;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckLCDSegment : Input, IInputTouchScreen, IInputTouchScreen.Drag, IInputDisplay {
	private StreamDeckDevice _device;

	public StreamDeckLCDSegment(StreamDeckDevice device, int x, int y, uint w, UInt2 displayResolution)
		: base(x, y, w, 1, InputIcon.TouchScreen) {
		_device = device;
		DisplayResolution = displayResolution;
	}

	public UInt2 DisplayResolution { get; }

	public Task ApplyImage(int key) => throw new NotImplementedException();

	public Task ClearImage() => throw new NotImplementedException();

	public event Action<Int2>? TouchScreenPressed;
	public event Action<Int2>? TouchScreenReleased;
	
	public event Action<Int2>? TouchScreenDragStart;
	public event Action<Int2>? TouchScreenDragging;
	public event Action<Int2>? TouchScreenDragEnd;

	internal void CallPressed(Int2 position) {
		TouchScreenPressed?.Invoke(position);
	}

	internal void CallReleased(Int2 position) {
		TouchScreenReleased?.Invoke(position);
	}
	
	internal void CallDrag(Int2 start, Int2 end) {
		TouchScreenDragStart?.Invoke(start);
		TouchScreenDragEnd?.Invoke(end);
	}
}