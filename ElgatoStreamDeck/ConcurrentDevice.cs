// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.PixelFormats;

namespace ElgatoStreamDeck;

public class ConcurrentDevice(Device device) : IDevice {
	private readonly Kind _kind = device.Kind();

	public Kind Kind() => _kind;

	public string Manufacturer() {
		lock (device) {
			return device.Manufacturer();
		}
	}

	public string Product() {
		lock (device) {
			return device.Product();
		}
	}

	public string SerialNumber() {
		lock (device) {
			return device.SerialNumber();
		}
	}

	public string FirmwareVersion() {
		lock (device) {
			return device.FirmwareVersion();
		}
	}

	public Input? ReadInput(int? timeout) {
		lock (device) {
			return device.ReadInput(timeout);
		}
	}

	public void Reset() {
		lock (device) {
			device.Reset();
		}
	}

	public void SetBrightness(byte percent) {
		lock (device) {
			device.SetBrightness(percent);
		}
	}

	public void WriteImage(byte keyIndex, ReadOnlySpan<byte> imageData) {
		lock (device) {
			device.WriteImage(keyIndex, imageData);
		}
	}

	public void WriteLcd(ushort x, ushort y, ushort w, ushort h, ReadOnlySpan<byte> imageData) {
		lock (device) {
			device.WriteLcd(x, y, w, h, imageData);
		}
	}

	public void ClearButtonImage(byte keyIndex) {
		lock (device) {
			device.ClearButtonImage(keyIndex);
		}
	}

	public void SetButtonImage(byte keyIndex, Image image) {
		lock (device) {
			device.SetButtonImage(keyIndex, image);
		}
	}

	public void SetButtonImage(byte keyIndex, Image<Rgb24> image) {
		lock (device) {
			device.SetButtonImage(keyIndex, image);
		}
	}

	public void SetButtonImage(byte keyIndex, ReadOnlySpan<byte> image, int width, int height) {
		lock (device) {
			device.SetButtonImage(keyIndex, image, width, height);
		}
	}

	public void Dispose() {
		device.Dispose();
		GC.SuppressFinalize(this);
	}
}