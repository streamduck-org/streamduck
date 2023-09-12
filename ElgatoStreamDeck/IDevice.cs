using System;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.PixelFormats;

namespace ElgatoStreamDeck;

public interface IDevice : IDisposable {
	Kind Kind();
	string Manufacturer();
	string Product();
	string SerialNumber();
	string FirmwareVersion();
	Input? ReadInput(int? timeout = null);
	void Reset();
	void SetBrightness(byte percent);
	void WriteImage(byte keyIndex, ReadOnlySpan<byte> imageData);
	void WriteLcd(ushort x, ushort y, ushort w, ushort h, ReadOnlySpan<byte> imageData);
	void ClearButtonImage(byte keyIndex);
	void SetButtonImage(byte keyIndex, Image image);
	void SetButtonImage(byte keyIndex, Image<Rgb24> image);
	void SetButtonImage(byte keyIndex, ReadOnlySpan<byte> image, int width, int height);
}