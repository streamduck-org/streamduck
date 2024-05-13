// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using HidApi;
using Microsoft.Extensions.Caching.Memory;
using Streamduck.Data;
using Streamduck.Devices;
using Streamduck.Interfaces;
using StreamduckStreamDeck.Inputs;
using Device = Streamduck.Devices.Device;
using ElgatoDevice = ElgatoStreamDeck.IDevice;
using Input = Streamduck.Inputs.Input;

namespace StreamduckStreamDeck;

public class StreamDeckDevice : Device, IDisposable, IConfigurable<StreamDeckDeviceOptions> {
	internal readonly ElgatoDevice _device;
	private readonly DeviceReader _deviceReader;
	internal readonly IMemoryCache _imageCache = new MemoryCache(new MemoryCacheOptions());

	internal readonly MemoryCacheEntryOptions _imageCacheEntryOptions = new() {
		SlidingExpiration = TimeSpan.FromMinutes(5)
	};

	private readonly Thread _readingThread;

	public StreamDeckDevice(ElgatoDevice device, DeviceIdentifier identifier) : base(identifier) {
		_device = device;
		_deviceReader = new DeviceReader(_device);

		// Creating input objects
		var kind = StreamDeckDriver.DescriptionToKind(identifier.Description);

		var inputs = new Input[kind.KeyCount() + kind.EncoderCount() + (kind.LcdStripSize() != null ? 1 : 0)];

		// Setting buttons
		var resolution = kind.KeyImageMode().Resolution;
		var hasScreen = resolution.Item1 > 0;

		for (var x = 0; x < kind.ColumnCount(); x++)
		for (var y = 0; y < kind.RowCount(); y++) {
			var i = x + y * kind.ColumnCount();
			inputs[i] = hasScreen
				? new StreamDeckButton(
					this,
					x,
					y,
					new UInt2(resolution.Item1, resolution.Item2),
					(byte)i
				)
				: new StreamDeckButtonWithoutDisplay(x, y);
		}

		// Setting screen
		if (kind.LcdStripSize() is { } strip)
			inputs[kind.KeyCount()] = new StreamDeckLCDSegment(
				this,
				0,
				kind.RowCount(),
				4,
				new UInt2(strip.Item1, strip.Item2)
			);

		// Setting encoders
		for (var i = 0; i < kind.EncoderCount(); i++)
			inputs[kind.KeyCount() + 1 + i] = new StreamDeckEncoder(i, kind.RowCount() + 1);

		Inputs = inputs;

		// Reading thread
		_readingThread = new Thread(ReaderThread);
		_readingThread.Start();
	}

	public override Input[] Inputs { get; }
	public StreamDeckDeviceOptions Config { get; set; } = new();

	public void Dispose() {
		Die();
		_readingThread.Interrupt();
		_device.Dispose();
		GC.SuppressFinalize(this);
	}

	private void ReaderThread() {
		var lcdIndex = _device.Kind().KeyCount();
		var encoderOffset = _device.Kind().KeyCount() + 1;

		while (Alive)
			try {
				foreach (var input in _deviceReader.Read())
					switch (input) {
						case DeviceReader.Input.ButtonPressed buttonPressed: {
							if (Inputs[buttonPressed.key] is StreamDeckButton button) button.CallPressed();

							break;
						}

						case DeviceReader.Input.ButtonReleased buttonReleased: {
							if (Inputs[buttonReleased.key] is StreamDeckButton button) button.CallReleased();

							break;
						}

						case DeviceReader.Input.EncoderPressed encoderPressed: {
							if (Inputs[encoderOffset + encoderPressed.encoder] is StreamDeckEncoder encoder)
								encoder.CallPressed();

							break;
						}

						case DeviceReader.Input.EncoderReleased encoderReleased: {
							if (Inputs[encoderOffset + encoderReleased.encoder] is StreamDeckEncoder encoder)
								encoder.CallReleased();

							break;
						}

						case DeviceReader.Input.EncoderTwist encoderTwist: {
							if (Inputs[encoderOffset + encoderTwist.encoder] is StreamDeckEncoder encoder)
								encoder.CallTwist(encoderTwist.value);

							break;
						}

						case DeviceReader.Input.TouchScreenPress touchScreenPress: {
							if (Inputs[lcdIndex] is StreamDeckLCDSegment segment) {
								segment.CallPressed(new Int2(touchScreenPress.X, touchScreenPress.Y));
								segment.CallReleased(new Int2(touchScreenPress.X, touchScreenPress.Y));
							}

							break;
						}

						case DeviceReader.Input.TouchScreenLongPress touchScreenPress: {
							if (Inputs[lcdIndex] is StreamDeckLCDSegment segment)
								Task.Run(
									async () => {
										segment.CallPressed(new Int2(touchScreenPress.X, touchScreenPress.Y));
										await Task.Delay(TimeSpan.FromSeconds(1));
										segment.CallReleased(new Int2(touchScreenPress.X, touchScreenPress.Y));
									}
								);

							break;
						}

						case DeviceReader.Input.TouchScreenSwipe touchScreenSwipe: {
							if (Inputs[lcdIndex] is StreamDeckLCDSegment segment)
								segment.CallDrag(
									new Int2(touchScreenSwipe.StartX, touchScreenSwipe.StartY),
									new Int2(touchScreenSwipe.EndX, touchScreenSwipe.EndY)
								);

							break;
						}
					}
			} catch (HidException e) {
				if (e.Message.Contains("Input/output error"))
					Die();
				else
					throw;
			}
	}

	internal void SetCache(long key, byte[] data) {
		_imageCache.Set(key, data, _imageCacheEntryOptions);
	}
}