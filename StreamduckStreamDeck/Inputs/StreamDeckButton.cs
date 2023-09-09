using System;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Microsoft.Extensions.Caching.Memory;
using SixLabors.ImageSharp;
using Streamduck.Definitions;
using Streamduck.Definitions.Inputs;
using Input = Streamduck.Definitions.Inputs.Input;

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
	public int AppendHashKey(int key) => $"{key}button".GetHashCode();

	public async Task UploadImage(int key, Image image) {
		var data = await ImageUtils.EncodeImageForButtonAsync(image, _device._device.Kind());
		_device.SetCache(key, data);
	}

	public ValueTask<bool> ApplyImage(int key) {
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