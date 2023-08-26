using System;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.PixelFormats;

namespace ElgatoStreamDeck;

public class ConcurrentDevice : IDevice {
	private readonly Device _device;
	private readonly Kind _kind;

	public ConcurrentDevice(Device device) {
		_device = device;
		_kind = device.Kind();
	}

	public Kind Kind() => _kind;

	public string Manufacturer() {
		lock (_device) {
			return _device.Manufacturer();
		}
	}

	public string Product() {
		lock (_device) {
			return _device.Product();
		}
	}

	public string SerialNumber() {
		lock (_device) {
			return _device.SerialNumber();
		}
	}

	public string FirmwareVersion() {
		lock (_device) {
			return _device.FirmwareVersion();
		}
	}

	public Input? ReadInput(int? timeout) {
		lock (_device) {
			return _device.ReadInput(timeout);
		}
	}

	public void Reset() {
		lock (_device) {
			_device.Reset();
		}
	}

	public void SetBrightness(byte percent) {
		lock (_device) {
			_device.SetBrightness(percent);
		}
	}

	public void WriteImage(byte keyIndex, ReadOnlySpan<byte> imageData) {
		lock (_device) {
			_device.WriteImage(keyIndex, imageData);
		}
	}

	public void WriteLcd(ushort x, ushort y, ushort w, ushort h, ReadOnlySpan<byte> imageData) {
		lock (_device) {
			_device.WriteLcd(x, y, w, h, imageData);
		}
	}

	public void ClearButtonImage(byte keyIndex) {
		lock (_device) {
			_device.ClearButtonImage(keyIndex);
		}
	}

	public void SetButtonImage(byte keyIndex, Image image) {
		lock (_device) {
			_device.SetButtonImage(keyIndex, image);
		}
	}

	public void SetButtonImage(byte keyIndex, Image<Rgb24> image) {
		lock (_device) {
			_device.SetButtonImage(keyIndex, image);
		}
	}

	public void SetButtonImage(byte keyIndex, ReadOnlySpan<byte> image, int width, int height) {
		lock (_device) {
			_device.SetButtonImage(keyIndex, image, width, height);
		}
	}
}