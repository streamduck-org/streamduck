// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Microsoft.Extensions.Caching.Memory;
using SixLabors.ImageSharp;
using Streamduck.Data;
using Streamduck.Inputs;
using Input = Streamduck.Inputs.Input;

namespace StreamduckStreamDeck.Inputs;

public class StreamDeckButton(StreamDeckDevice device, int x, int y, UInt2 displayResolution, byte keyIndex)
	: Input(x, y, 1, 1, InputIcon.Button), IInputButton, IInputDisplay {
	public event Action? ButtonPressed;
	public event Action? ButtonReleased;

	public UInt2 DisplayResolution { get; } = displayResolution;

	public long AppendHashKey(long key) => $"{key}button".GetHashCode();

	public async Task UploadImage(long key, Image image) {
		device.ThrowDisconnectedIfDead();
		var data = await ImageUtils.EncodeImageForButtonAsync(image, device._device.Kind());
		device.SetCache(key, data);
	}

	public ValueTask<bool> ApplyImage(long key) {
		device.ThrowDisconnectedIfDead();
		if (!device._imageCache.TryGetValue(key, out byte[]? data))
			return ValueTask.FromResult(false);

		device._device.WriteImage(keyIndex, data);

		return ValueTask.FromResult(true);
	}

	internal void CallPressed() {
		ButtonPressed?.Invoke();
	}

	internal void CallReleased() {
		ButtonReleased?.Invoke();
	}
}