using System;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Microsoft.Extensions.Caching.Memory;
using SixLabors.ImageSharp;
using Streamduck.Data;
using Streamduck.Inputs;
using Input = Streamduck.Inputs.Input;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckLCDSegment(StreamDeckDevice device, int x, int y, uint w, UInt2 displayResolution)
	: Input(x, y, w, 1, InputIcon.TouchScreen), IInputTouchScreen, IInputTouchScreen.Drag, IInputDisplay {
	public event Action<Int2>? TouchScreenDragStart;
	public event Action<Int2>? TouchScreenDragEnd;

	public UInt2 DisplayResolution { get; } = displayResolution;
	public long AppendHashKey(long key) => $"{key}lcd".GetHashCode();

	public async Task UploadImage(long key, Image image) {
		device.ThrowDisconnectedIfDead();
		var data = await ImageUtils.EncodeImageForLcdAsync(image, image.Width, image.Height);
		device.SetCache(key, data);
	}

	public ValueTask<bool> ApplyImage(long key) {
		device.ThrowDisconnectedIfDead();
		device._imageCache.TryGetValue(key, out byte[]? data);

		if (data == null) return ValueTask.FromResult(false);

		var resolution = device._device.Kind().KeyImageMode().Resolution;
		device._device.WriteLcd(0, 0, (ushort)resolution.Item1, (ushort)resolution.Item2, data);

		return ValueTask.FromResult(true);
	}

	public event Action<Int2>? TouchScreenPressed;
	public event Action<Int2>? TouchScreenReleased;

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