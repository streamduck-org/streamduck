using System;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Microsoft.Extensions.Caching.Memory;
using SixLabors.ImageSharp;
using Streamduck.Definitions;
using Streamduck.Definitions.Inputs;
using Input = Streamduck.Definitions.Inputs.Input;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckLCDSegment : Input, IInputTouchScreen, IInputTouchScreen.Drag, IInputDisplay {
	private readonly StreamDeckDevice _device;

	public StreamDeckLCDSegment(StreamDeckDevice device, int x, int y, uint w, UInt2 displayResolution)
		: base(x, y, w, 1, InputIcon.TouchScreen) {
		_device = device;
		DisplayResolution = displayResolution;
	}

	public UInt2 DisplayResolution { get; }
	public int AppendHashKey(int key) => $"{key}lcd".GetHashCode();

	public async Task UploadImage(int key, Image image) {
		var data = await ImageUtils.EncodeImageForLcdAsync(image, image.Width, image.Height);
		_device.SetCache(key, data);
	}

	public ValueTask<bool> ApplyImage(int key) {
		_device._imageCache.TryGetValue(key, out byte[]? data);

		if (data == null) return ValueTask.FromResult(false);
		
		var resolution = _device._device.Kind().KeyImageMode().Resolution;
		_device._device.WriteLcd(0, 0, (ushort)resolution.Item1, (ushort)resolution.Item2, data);

		return ValueTask.FromResult(true);
	}

	public event Action<Int2>? TouchScreenPressed;
	public event Action<Int2>? TouchScreenReleased;
	public event Action<Int2>? TouchScreenDragStart;
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