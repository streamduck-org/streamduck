// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using HidApi;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.PixelFormats;

namespace ElgatoStreamDeck;

public class Device : IDevice {
	private readonly MemoryStream _buffer;
	private readonly HidApi.Device _device;
	private readonly Kind _kind;
	private readonly byte[] _packetData;

	public Device(HidApi.Device device, Kind kind) {
		_device = device;
		_kind = kind;
		_packetData = new byte[ImageReportLength()];
		_buffer = new MemoryStream(_packetData);
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
		try {
			if (_kind == ElgatoStreamDeck.Kind.Plus) {
				var length = Math.Max(14, 5 + _kind.EncoderCount());
				var data = timeout != null ? _device.ReadTimeout(length, timeout.Value) : _device.Read(length);

				if (data.Length <= 0 || data[0] == 0) return null;

				return data[1] switch {
					0 => new Input.ButtonStateChange(ReadButtonStates(data).ToArray()),
					2 => ReadLcdInput(data),
					3 => ReadEncoderInput(data),
					_ => throw new InvalidDataException("Bad data sent by the device")
				};
			} else {
				var length = _kind switch {
					ElgatoStreamDeck.Kind.Original or
						ElgatoStreamDeck.Kind.Mini or
						ElgatoStreamDeck.Kind.MiniMk2 => 1 + _kind.KeyCount(),
					_ => 4 + _kind.KeyCount()
				};

				var data = timeout != null ? _device.ReadTimeout(length, timeout.Value) : _device.Read(length);

				return data.Length <= 0 || data[0] == 0
					? null
					: new Input.ButtonStateChange(ReadButtonStates(data).ToArray());
			}
		} catch (HidException e) {
			if (e.Message.Contains("Interrupted system call")) return null;

			throw;
		}
	}

	public void Reset() {
		switch (_kind) {
			case ElgatoStreamDeck.Kind.Original:
			case ElgatoStreamDeck.Kind.Mini:
			case ElgatoStreamDeck.Kind.MiniMk2: {
				_device.SendFeatureReport(new byte[] {
					0x0B, 0x63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
				});

				break;
			}

			case ElgatoStreamDeck.Kind.OriginalV2:
			case ElgatoStreamDeck.Kind.Xl:
			case ElgatoStreamDeck.Kind.XlV2:
			case ElgatoStreamDeck.Kind.Mk2:
			case ElgatoStreamDeck.Kind.Pedal:
			case ElgatoStreamDeck.Kind.Plus: {
				_device.SendFeatureReport(new byte[] {
					0x03, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0
				});

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
				_device.SendFeatureReport(new byte[] {
					0x05, 0x55, 0xaa, 0xd1, 0x01, percent, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
				});

				break;
			}

			case ElgatoStreamDeck.Kind.OriginalV2:
			case ElgatoStreamDeck.Kind.Xl:
			case ElgatoStreamDeck.Kind.XlV2:
			case ElgatoStreamDeck.Kind.Mk2:
			case ElgatoStreamDeck.Kind.Pedal:
			case ElgatoStreamDeck.Kind.Plus: {
				_device.SendFeatureReport(new byte[] {
					0x03, 0x08, percent, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0, 0, 0
				});

				break;
			}

			case ElgatoStreamDeck.Kind.Unknown:
			default:
				throw new ArgumentOutOfRangeException();
		}
	}

	public void WriteImage(byte keyIndex, ReadOnlySpan<byte> imageData) {
		if (keyIndex >= _kind.KeyCount()) throw new ArgumentOutOfRangeException(nameof(keyIndex));

		var key = _kind switch {
			ElgatoStreamDeck.Kind.Original => FlipKeyIndex(keyIndex),
			_ => keyIndex
		};

		if (!_kind.IsVisual()) throw new InvalidOperationException("Device doesn't have a screen");

		var imageReportHeaderLength = _kind switch {
			ElgatoStreamDeck.Kind.Original or ElgatoStreamDeck.Kind.Mini or ElgatoStreamDeck.Kind.MiniMk2 => 16,
			_ => 8
		};

		var imageReportPayloadLength = _kind switch {
			ElgatoStreamDeck.Kind.Original => imageData.Length / 2,
			_ => ImageReportLength() - imageReportHeaderLength
		};

		var pageNumber = 0;
		var bytesRemaining = imageData.Length;

		while (bytesRemaining > 0) {
			var thisLength = Math.Min(bytesRemaining, imageReportPayloadLength);
			var bytesSent = pageNumber * imageReportPayloadLength;

			_buffer.Position = 0;

			// Writing header
			switch (_kind) {
				case ElgatoStreamDeck.Kind.Original:
					_buffer.Write(new byte[] {
						0x02,
						0x01,
						(byte)(pageNumber + 1),
						0,
						(byte)(thisLength == bytesRemaining ? 1 : 0),
						(byte)(key + 1),
						0,
						0,
						0,
						0,
						0,
						0,
						0,
						0,
						0,
						0
					});
					break;
				case ElgatoStreamDeck.Kind.Mini:
				case ElgatoStreamDeck.Kind.MiniMk2:
					_buffer.Write(new byte[] {
						0x02,
						0x01,
						(byte)pageNumber,
						0,
						(byte)(thisLength == bytesRemaining ? 1 : 0),
						(byte)(key + 1),
						0,
						0,
						0,
						0,
						0,
						0,
						0,
						0,
						0,
						0
					});
					break;

				case ElgatoStreamDeck.Kind.OriginalV2:
				case ElgatoStreamDeck.Kind.Xl:
				case ElgatoStreamDeck.Kind.XlV2:
				case ElgatoStreamDeck.Kind.Mk2:
				case ElgatoStreamDeck.Kind.Pedal:
				case ElgatoStreamDeck.Kind.Plus:
					_buffer.Write(new byte[] {
						0x02,
						0x07,
						key,
						(byte)(thisLength == bytesRemaining ? 1 : 0),
						(byte)(thisLength & 0xff),
						(byte)(thisLength >> 8),
						(byte)(pageNumber & 0xff),
						(byte)(pageNumber >> 8)
					});
					break;

				case ElgatoStreamDeck.Kind.Unknown:
				default:
					throw new ArgumentOutOfRangeException();
			}

			// Writing image
			_buffer.Write(imageData[bytesSent .. (bytesSent + thisLength)]);

			_device.Write(_packetData);

			bytesRemaining -= thisLength;
			pageNumber++;
		}
	}

	public void WriteLcd(ushort x, ushort y, ushort w, ushort h, ReadOnlySpan<byte> imageData) {
		if (_kind != ElgatoStreamDeck.Kind.Plus)
			throw new InvalidOperationException("Device doesn't have an LCD screen");

		const int imageReportHeaderLength = 16;
		var imageReportPayloadLength = ImageReportLength() - imageReportHeaderLength;

		var pageNumber = 0;
		var bytesRemaining = imageData.Length;

		while (bytesRemaining > 0) {
			var thisLength = Math.Min(bytesRemaining, imageReportPayloadLength);
			var bytesSent = pageNumber * imageReportPayloadLength;

			_buffer.Position = 0;

			// Writing header
			_buffer.Write(new byte[] {
				0x02,
				0x0c,
				(byte)(x & 0xff),
				(byte)(x >> 8),
				(byte)(y & 0xff),
				(byte)(y >> 8),
				(byte)(w & 0xff),
				(byte)(w >> 8),
				(byte)(h & 0xff),
				(byte)(h >> 8),
				(byte)(bytesRemaining <= imageReportPayloadLength ? 1 : 0),
				(byte)(pageNumber & 0xff),
				(byte)(pageNumber >> 8),
				(byte)(thisLength & 0xff),
				(byte)(thisLength >> 8),
				0
			});

			// Writing image
			_buffer.Write(imageData[bytesSent .. (bytesSent + thisLength)]);

			_device.Write(_packetData);

			bytesRemaining -= thisLength;
			pageNumber++;
		}
	}

	public void ClearButtonImage(byte keyIndex) {
		WriteImage(keyIndex, _kind.BlankImage());
	}

	public void SetButtonImage(byte keyIndex, Image image) {
		var imageData = ImageUtils.EncodeImageForButton(image, _kind);
		WriteImage(keyIndex, imageData);
	}

	public void SetButtonImage(byte keyIndex, Image<Rgb24> image) {
		var imageData = ImageUtils.EncodeImageForButton(image, _kind);
		WriteImage(keyIndex, imageData);
	}

	public void SetButtonImage(byte keyIndex, ReadOnlySpan<byte> image, int width, int height) {
		var imageData = ImageUtils.EncodeImageForButton(image, width, height, _kind);
		WriteImage(keyIndex, imageData);
	}

	public void Dispose() {
		_buffer.Dispose();
		_device.Dispose();
		GC.SuppressFinalize(this);
	}

	private int ImageReportLength() => _kind switch {
		ElgatoStreamDeck.Kind.Original => 8191,
		_ => 1024
	};

	private byte FlipKeyIndex(byte key) {
		var col = (byte)(key % _kind.ColumnCount());
		return (byte)(key - col + (_kind.ColumnCount() - 1 - col));
	}

	private IEnumerable<bool> ReadButtonStates(ReadOnlySpan<byte> data) {
		var values = new bool[_kind.KeyCount()];

		switch (_kind) {
			case ElgatoStreamDeck.Kind.Original: {
				for (var i = 0; i < _kind.KeyCount(); i++) {
					var flipped = FlipKeyIndex((byte)i) + 1;
					values[i] = data[flipped] != 0;
				}

				break;
			}

			case ElgatoStreamDeck.Kind.Mini:
			case ElgatoStreamDeck.Kind.MiniMk2: {
				for (var i = 0; i < _kind.KeyCount(); i++) {
					values[i] = data[1 + i] != 0;
				}

				break;
			}

			case ElgatoStreamDeck.Kind.OriginalV2:
			case ElgatoStreamDeck.Kind.Xl:
			case ElgatoStreamDeck.Kind.XlV2:
			case ElgatoStreamDeck.Kind.Mk2:
			case ElgatoStreamDeck.Kind.Pedal:
			case ElgatoStreamDeck.Kind.Plus: {
				for (var i = 0; i < _kind.KeyCount(); i++) {
					values[i] = data[4 + i] != 0;
				}

				break;
			}

			case ElgatoStreamDeck.Kind.Unknown:
			default:
				throw new ArgumentOutOfRangeException();
		}

		return values;
	}

	private static Input ReadLcdInput(ReadOnlySpan<byte> data) {
		var startX = BitConverter.ToUInt16(data[6..8]);
		var startY = BitConverter.ToUInt16(data[8..10]);

		switch (data[4]) {
			case 1: {
				return new Input.TouchScreenPress(startX, startY);
			}

			case 2: {
				return new Input.TouchScreenLongPress(startX, startY);
			}

			case 3: {
				var endX = BitConverter.ToUInt16(data[10..12]);
				var endY = BitConverter.ToUInt16(data[12..14]);

				return new Input.TouchScreenSwipe(startX, startY, endX, endY);
			}

			default:
				throw new InvalidDataException("Bad data sent by the device");
		}
	}

	private IEnumerable<bool> ReadEncoderState(ReadOnlySpan<byte> data) {
		var values = new bool[_kind.EncoderCount()];

		for (var i = 0; i < _kind.EncoderCount(); i++) {
			values[i] = data[i + 5] != 0;
		}

		return values;
	}

	private IEnumerable<sbyte> ReadEncoderTwist(ReadOnlySpan<byte> data) {
		var values = new sbyte[_kind.EncoderCount()];

		for (var i = 0; i < _kind.EncoderCount(); i++) {
			values[i] = (sbyte)data[i + 5];
		}

		return values;
	}

	private Input ReadEncoderInput(ReadOnlySpan<byte> data) {
		return data[4] switch {
			0 => new Input.EncoderStateChange(ReadEncoderState(data).ToArray()),
			1 => new Input.EncoderTwist(ReadEncoderTwist(data).ToArray()),
			_ => throw new InvalidDataException("Bad data sent by the device")
		};
	}
}