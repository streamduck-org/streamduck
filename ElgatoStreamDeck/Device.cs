using System;
using System.IO;
using System.Text;
using ElgatoStreamDeck.Inputs;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.PixelFormats;

namespace ElgatoStreamDeck;

public class Device : IDevice {
	private readonly HidApi.Device _device;
	private readonly Kind _kind;

	public Device(HidApi.Device device, Kind kind) {
		_device = device;
		_kind = kind;
	}

	public Kind Kind() => _kind;

	public string Manufacturer() => _device.GetManufacturer();

	public string Product() => _device.GetProduct();

	public string SerialNumber() {
		switch (_kind) {
			case ElgatoStreamDeck.Kind.Original:
			case ElgatoStreamDeck.Kind.Mini:
				return Encoding.UTF8.GetString(_device.GetFeatureReport(0x03, 17)[5..]);

			case ElgatoStreamDeck.Kind.MiniMk2:
				return Encoding.UTF8.GetString(_device.GetFeatureReport(0x03, 32)[5..]);

			case ElgatoStreamDeck.Kind.OriginalV2:
			case ElgatoStreamDeck.Kind.Xl:
			case ElgatoStreamDeck.Kind.XlV2:
			case ElgatoStreamDeck.Kind.Mk2:
			case ElgatoStreamDeck.Kind.Pedal:
			case ElgatoStreamDeck.Kind.Plus:
				return Encoding.UTF8.GetString(_device.GetFeatureReport(0x06, 32)[2..]);

			case ElgatoStreamDeck.Kind.Unknown:
			default:
				throw new ArgumentOutOfRangeException();
		}
	}

	public string FirmwareVersion() {
		switch (_kind) {
			case ElgatoStreamDeck.Kind.Original:
			case ElgatoStreamDeck.Kind.Mini:
			case ElgatoStreamDeck.Kind.MiniMk2:
				return Encoding.UTF8.GetString(_device.GetFeatureReport(0x04, 17)[5..]);

			case ElgatoStreamDeck.Kind.OriginalV2:
			case ElgatoStreamDeck.Kind.Xl:
			case ElgatoStreamDeck.Kind.XlV2:
			case ElgatoStreamDeck.Kind.Mk2:
			case ElgatoStreamDeck.Kind.Pedal:
			case ElgatoStreamDeck.Kind.Plus:
				return Encoding.UTF8.GetString(_device.GetFeatureReport(0x05, 32)[6..]);

			case ElgatoStreamDeck.Kind.Unknown:
			default:
				throw new ArgumentOutOfRangeException();
		}
	}

	public Input? ReadInput(int? timeout = null) {
		if (_kind == ElgatoStreamDeck.Kind.Plus) {
			var length = Math.Max(14, 5 + _kind.EncoderCount());
			var data = timeout != null ? _device.ReadTimeout(length, timeout.Value) : _device.Read(length);
		} else { }

		return null;
	}

	public void Reset() {
		switch (_kind) {
			case ElgatoStreamDeck.Kind.Original:
			case ElgatoStreamDeck.Kind.Mini:
			case ElgatoStreamDeck.Kind.MiniMk2: {
				using var buffer = new MemoryStream();
				buffer.Write(new byte[] { 0x0B, 0x63 });
				buffer.Write(new byte[15]);
				_device.SendFeatureReport(buffer.ToArray());

				break;
			}

			case ElgatoStreamDeck.Kind.OriginalV2:
			case ElgatoStreamDeck.Kind.Xl:
			case ElgatoStreamDeck.Kind.XlV2:
			case ElgatoStreamDeck.Kind.Mk2:
			case ElgatoStreamDeck.Kind.Pedal:
			case ElgatoStreamDeck.Kind.Plus: {
				using var buffer = new MemoryStream();
				buffer.Write(new byte[] { 0x03, 0x02 });
				buffer.Write(new byte[30]);
				_device.SendFeatureReport(buffer.ToArray());

				break;
			}

			case ElgatoStreamDeck.Kind.Unknown:
			default:
				throw new ArgumentOutOfRangeException();
		}
	}

	public void SetBrightness(byte percent) {
		switch (_kind) {
			case ElgatoStreamDeck.Kind.Original:
			case ElgatoStreamDeck.Kind.Mini:
			case ElgatoStreamDeck.Kind.MiniMk2: {
				using var buffer = new MemoryStream();
				buffer.Write(new byte[] {
					0x05,
					0x55,
					0xaa,
					0xd1,
					0x01,
					percent
				});
				buffer.Write(new byte[11]);
				_device.SendFeatureReport(buffer.ToArray());

				break;
			}

			case ElgatoStreamDeck.Kind.OriginalV2:
			case ElgatoStreamDeck.Kind.Xl:
			case ElgatoStreamDeck.Kind.XlV2:
			case ElgatoStreamDeck.Kind.Mk2:
			case ElgatoStreamDeck.Kind.Pedal:
			case ElgatoStreamDeck.Kind.Plus: {
				using var buffer = new MemoryStream();
				buffer.Write(new byte[] {
					0x03,
					0x08,
					percent
				});
				buffer.Write(new byte[29]);
				_device.SendFeatureReport(buffer.ToArray());

				break;
			}

			case ElgatoStreamDeck.Kind.Unknown:
			default:
				throw new ArgumentOutOfRangeException();
		}
	}

	public void WriteImage(byte keyIndex, byte[] imageData) {
		throw new NotImplementedException();
	}

	public void WriteLcd(ushort x, ushort y, ushort w, ushort h, byte[] imageData) {
		throw new NotImplementedException();
	}

	public void ClearButtonImage(byte keyIndex) {
		throw new NotImplementedException();
	}

	public void SetButtonImage(byte keyIndex, Image image) {
		throw new NotImplementedException();
	}

	public void SetButtonImage(byte keyIndex, Image<Rgb24> image) {
		throw new NotImplementedException();
	}

	public void SetButtonImage(byte keyIndex, byte[] image, int width, int height) {
		throw new NotImplementedException();
	}
}