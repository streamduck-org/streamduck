using System;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Microsoft.Extensions.Caching.Memory;
using SixLabors.ImageSharp;
using Streamduck.Data;
using Streamduck.Inputs;
using Input = Streamduck.Inputs.Input;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckButton : Input, IInputButton, IInputDisplay {
	private readonly StreamDeckDevice _device;
	private readonly byte _keyIndex;

	public StreamDeckButton(StreamDeckDevice device, int x, int y, UInt2 displayResolution, byte keyIndex)
		: base(x, y, 1, 1, InputIcon.Button) {
		_device = device;
		DisplayResolution = displayResolution;
		_keyIndex = keyIndex;
	}

	public event Action? ButtonPressed;
	public event Action? ButtonReleased;

	public UInt2 DisplayResolution { get; }
	public long AppendHashKey(long key) => $"{key}button".GetHashCode();

	public async Task UploadImage(long key, Image image) {
		_device.ThrowDisconnectedIfDead();
		var data = await ImageUtils.EncodeImageForButtonAsync(image, _device._device.Kind());
		_device.SetCache(key, data);
	}

	public ValueTask<bool> ApplyImage(long key) {
		_device.ThrowDisconnectedIfDead();
		if (!_device._imageCache.TryGetValue(key, out byte[]? data))
			return ValueTask.FromResult(false);

		_device._device.WriteImage(_keyIndex, data);

		return ValueTask.FromResult(true);
	}

	internal void CallPressed() {
		ButtonPressed?.Invoke();
	}

	internal void CallReleased() {
		ButtonReleased?.Invoke();
	}
}